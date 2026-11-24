obj-m += panic_lab.o
panic_lab-y := src/panic_lab_main.o src/faults.o

ccflags-y += -I$(src)/src

KDIR ?= /lib/modules/$(shell uname -r)/build

all:
	$(MAKE) -C $(KDIR) M=$(PWD) modules

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean
