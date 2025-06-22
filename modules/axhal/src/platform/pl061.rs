use tock_registers::{registers::{ReadWrite, ReadOnly, WriteOnly}, register_structs, register_bitfields};

register_bitfields![
    u32,
    pub GPIOIE [
        Pin0 OFFSET(0) NUMBITS(1) [
            clear = 0,
            set = 1
        ],
        Pin1 OFFSET(1) NUMBITS(1) [
            clear = 0,
            set = 1
        ],
        Pin2 OFFSET(2) NUMBITS(1) [
            clear = 0,
            set = 1
        ],
        Pin3 OFFSET(3) NUMBITS(1) [
            clear = 0,
            set = 1
        ],
        Pin4 OFFSET(4) NUMBITS(1) [
            clear = 0,
            set = 1
        ],
        Pin5 OFFSET(5) NUMBITS(1) [
            clear = 0,
            set = 1
        ],
        Pin6 OFFSET(6) NUMBITS(1) [
            clear = 0,
            set = 1
        ],
        Pin7 OFFSET(7) NUMBITS(1) [
            clear = 0,
            set = 1
        ]
    ],
    pub GPIOIC [
        Pin0 OFFSET(0) NUMBITS(1) [
            set = 1
        ],
        Pin1 OFFSET(1) NUMBITS(1) [
            set = 1
        ],
        Pin2 OFFSET(2) NUMBITS(1) [
            set = 1
        ],
        Pin3 OFFSET(3) NUMBITS(1) [
            set = 1
        ],
        Pin4 OFFSET(4) NUMBITS(1) [
            set = 1
        ],
        Pin5 OFFSET(5) NUMBITS(1) [
            set = 1
        ],
        Pin6 OFFSET(6) NUMBITS(1) [
            set = 1
        ],
        Pin7 OFFSET(7) NUMBITS(1) [
            set = 1
        ]
    ]
];

register_structs! {
    pub PL061Regs {
        (0x000 => pub data: ReadWrite<u32>),
        (0x004 => __reserved_0),
        (0x400 => pub dir: ReadWrite<u32>),
        (0x404 => pub is: ReadWrite<u32>),
        (0x408 => pub ibe: ReadWrite<u32>),
        (0x40C => pub iev: ReadWrite<u32>),
        (0x410 => pub ie: ReadWrite<u32, GPIOIE::Register>),
        (0x414 => pub ris: ReadOnly<u32>),
        (0x418 => pub mis: ReadOnly<u32>),
        (0x41C => pub ic: WriteOnly<u32, GPIOIC::Register>),
        (0x420 => pub afsel: ReadWrite<u32>),
        (0x424 => @END),
    }
}