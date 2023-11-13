
::udi::define_pio_ops!{pub RESET =
	END_IMM 0;
}

::udi::define_pio_ops!{pub ENABLE =
    END_IMM 0;
}
::udi::define_pio_ops!{pub DISBALE =
    END_IMM 0;
}

::udi::define_pio_ops!{pub IRQACK =
        // Entrypoint 0: Enable interrupts
        END_IMM 0;
        // 1: Normal
        LABEL 1;
        END_IMM 0;
        // 2: Overrun
        LABEL 2;
        // 3: Overrun irqs
        LABEL 3;
        END_IMM 0;
}