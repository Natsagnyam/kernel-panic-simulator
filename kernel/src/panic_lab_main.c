#include "faults.h"
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/kobject.h>
#include <linux/sysfs.h>
#include <linux/string.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("NATSKA");
MODULE_DESCRIPTION("Kernel Panic Detector and Fault Injection Lab (C-first)");


static struct kobject *panic_lab_kobj;
static char last_fault[64] = "none";

static ssize_t trigger_show(struct kobject *kobj,
			    struct kobj_attribute *attr, char *buf)
{
    return sysfs_emit(buf,
		      "last_fault=%s\n"
		      "commands: memleak | race | divzero | badptr\n",
		      last_fault);
}

static ssize_t trigger_store(struct kobject *kobj,
			     struct kobj_attribute *attr,
			     const char *buf, size_t count)
{
    char cmd[64];
    size_t len = min(count, sizeof(cmd) - 1);

    memcpy(cmd, buf, len);
    cmd[len] = '\0';
    strim(cmd);

    if (sysfs_streq(cmd, "memleak")) {
        strscpy(last_fault, "memleak", sizeof(last_fault));
        trigger_memory_leak();
    } else if (sysfs_streq(cmd, "race")) {
        strscpy(last_fault, "race", sizeof(last_fault));
        trigger_race_condition();
    } else if (sysfs_streq(cmd, "divzero")) {
        strscpy(last_fault, "divzero", sizeof(last_fault));
        (void)trigger_divide_by_zero(42, 0);
    } else if (sysfs_streq(cmd, "badptr")) {
        strscpy(last_fault, "badptr", sizeof(last_fault));
        trigger_bad_pointer();
    } else {
        pr_warn("panic_lab: unknown trigger command '%s'\n", cmd);
        return -EINVAL;
    }

    return count;
}

static ssize_t metrics_show(struct kobject *kobj,
			    struct kobj_attribute *attr, char *buf)
{
    s64 leaks = 0, bytes = 0, races = 0, divzeros = 0, badptrs = 0;

    get_fault_metrics(&leaks, &bytes, &races, &divzeros, &badptrs);

    return sysfs_emit(buf,
		      "leak_count=%lld\n"
		      "leaked_bytes=%lld\n"
		      "race_count=%lld\n"
		      "divzero_count=%lld\n"
		      "badptr_count=%lld\n",
		      (long long)leaks,
		      (long long)bytes,
		      (long long)races,
		      (long long)divzeros,
		      (long long)badptrs);
}

static struct kobj_attribute trigger_attr =
    __ATTR(trigger, 0660, trigger_show, trigger_store);

static struct kobj_attribute metrics_attr =
    __ATTR_RO(metrics);

static int __init panic_lab_init(void)
{
    int ret;

    panic_lab_kobj = kobject_create_and_add("panic_lab", kernel_kobj);
    if (!panic_lab_kobj) {
        pr_err("panic_lab: failed to create sysfs kobject\n");
        return -ENOMEM;
    }

    ret = sysfs_create_file(panic_lab_kobj, &trigger_attr.attr);
    if (ret) {
        pr_err("panic_lab: failed to create trigger sysfs file (%d)\n", ret);
        kobject_put(panic_lab_kobj);
        return ret;
    }

    ret = sysfs_create_file(panic_lab_kobj, &metrics_attr.attr);
    if (ret) {
        pr_err("panic_lab: failed to create metrics sysfs file (%d)\n", ret);
        sysfs_remove_file(panic_lab_kobj, &trigger_attr.attr);
        kobject_put(panic_lab_kobj);
        return ret;
    }

    pr_info("panic_lab: module loaded (/sys/kernel/panic_lab/{trigger,metrics})\n");
    return 0;
}

static void __exit panic_lab_exit(void)
{
    if (panic_lab_kobj) {
        sysfs_remove_file(panic_lab_kobj, &metrics_attr.attr);
        sysfs_remove_file(panic_lab_kobj, &trigger_attr.attr);
        kobject_put(panic_lab_kobj);
    }
    pr_info("panic_lab: module unloaded\n");
}

module_init(panic_lab_init);
module_exit(panic_lab_exit);
