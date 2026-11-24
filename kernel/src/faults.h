#ifndef PANIC_LAB_FAULTS_H
#define PANIC_LAB_FAULTS_H

#include <linux/types.h>

void trigger_memory_leak(void);
void trigger_race_condition(void);
int trigger_divide_by_zero(int numerator, int denominator);
void trigger_bad_pointer(void);
void get_fault_metrics(s64 *out_leak_count,
                       s64 *out_leaked_bytes,
                       s64 *out_race_count,
                       s64 *out_divzero_count,
                       s64 *out_badptr_count);

#endif /* PANIC_LAB_FAULTS_H */
