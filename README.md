# rustcore - a simple RISC-V core implementation
## Extensions implemented
Currently, no extensions are implemented. See the [roadmap](#roadmap)
## Roadmap
Extensions will be initially implemented in the order in which they are specified.
For now, only the unprivileged specification will be worked on. The privileged spec
may be implemented at a later date. [Official specifications].

The current plan is to implement RV32I, "Zifencei", RV64I and RV128I. Others may begin
development after these base instruction sets.

There is no plan to implement RV32E as it is meant for embedded systems. The
only difference between it and RV32I is a reduced amount of registers, down from
32 to a total of 16.

[Official specifications]: https://riscv.org/technical/specifications/

---

This project is a personal hobby of mine. Pull requests are appreciated.
More specific contribution guidelines will be available sooner or later.
CI and unit testing will be set up once the project grows more.

---

The whole project is licensed under the terms of the [GNU LGPL v3](./LICENSE)
(LGPL-3.0-or-later)
