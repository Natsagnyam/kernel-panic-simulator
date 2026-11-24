#include "faults.h"
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/slab.h>
#include <linux/kthread.h>
#include <linux/delay.h>
#include <linux/atomic.h>

/* Shared unprotected counter for the race condition test */
static volatile unsigned long shared_race_counter;

/* Live telemetry counters */
static atomic64_t leak_count = ATOMIC64_INIT(0);
static atomic64_t leaked_bytes = ATOMIC64_INIT(0);
static atomic64_t race_count = ATOMIC64_INIT(0);
static atomic64_t divzero_count = ATOMIC64_INIT(0);
static atomic64_t badptr_count = ATOMIC64_INIT(0);

/* 1. Memory leak trigger */
void trigger_memory_leak(void)
{
    void *leaked = kmalloc(4096, GFP_KERNEL);

    atomic64_inc(&leak_count);
    atomic64_add(4096, &leaked_bytes);
    pr_warn("panic_lab: leaked 4096 bytes at %p (total=%lld bytes, count=%lld)\n",
            leaked,
            (long long)atomic64_read(&leaked_bytes),
            (long long)atomic64_read(&leak_count));
}

/* 2. Race condition trigger (two threads hammering shared_race_counter) */
static int race_worker_fn(void *data)
{
    int iter;
    int id = (int)(long)data;

    for (iter = 0; iter < 500000; iter++) {
        unsigned long tmp = shared_race_counter;
        tmp += 1;
        shared_race_counter = tmp;
        if ((iter & 0x3fff) == 0)
            cond_resched();
    }

    pr_warn("panic_lab: race worker %d finished, counter=%lu\n", id, shared_race_counter);
    return 0;
}

void trigger_race_condition(void)
{
    struct task_struct *t1;
    struct task_struct *t2;

    atomic64_inc(&race_count);
    shared_race_counter = 0;
    pr_warn("panic_lab: starting unsynchronized race condition threads\n");

    t1 = kthread_run(race_worker_fn, (void *)1L, "panic_race_1");
    t2 = kthread_run(race_worker_fn, (void *)2L, "panic_race_2");

    if (IS_ERR(t1))
        pr_err("panic_lab: failed to start thread 1 (%ld)\n", PTR_ERR(t1));
    if (IS_ERR(t2))
        pr_err("panic_lab: failed to start thread 2 (%ld)\n", PTR_ERR(t2));
}

/* 3. Divide by zero trigger */
int trigger_divide_by_zero(int numerator, int denominator)
{
    volatile int num = numerator;
    volatile int den = denominator;

    atomic64_inc(&divzero_count);
    pr_emerg("panic_lab: executing integer division %d / %d\n", num, den);
    return num / den;
}

/* 4. Invalid / out-of-bounds pointer access trigger */
void trigger_bad_pointer(void)
{
    volatile int *bad_ptr = (volatile int *)0xdeadbeefULL;

    atomic64_inc(&badptr_count);
    pr_emerg("panic_lab: dereferencing invalid pointer %p\n", bad_ptr);
    (void)*bad_ptr;
}

/* Snapshot live metrics for sysfs */
void get_fault_metrics(s64 *out_leak_count,
                       s64 *out_leaked_bytes,
                       s64 *out_race_count,
                       s64 *out_divzero_count,
                       s64 *out_badptr_count)
{
    if (out_leak_count)
        *out_leak_count = atomic64_read(&leak_count);
    if (out_leaked_bytes)
        *out_leaked_bytes = atomic64_read(&leaked_bytes);
    if (out_race_count)
        *out_race_count = atomic64_read(&race_count);
    if (out_divzero_count)
        *out_divzero_count = atomic64_read(&divzero_count);
    if (out_badptr_count)
        *out_badptr_count = atomic64_read(&badptr_count);
}
