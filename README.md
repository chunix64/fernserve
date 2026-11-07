# fernserve
Embedded version of miniserve, serving files over HTTP

# Todos
## Short term goals:
- [ ] Replace all `alloc` structs and methods to `heapless`.
- [ ] Fix all warnings.
- [ ] Use macros for `AnyFs` (highly repetitive) or find other ways to implement polymorphism with `async`, `rtos` without sacrificing runtime performance.
- [ ] Implement the basic webui and release `v.0.0.1-beta`

## Long term goals:
- [ ] Implement `fatfs` and generalize the FileSystem abstract for `InternalStorage` and `ExternalStorage` with NOR flashes and NAND flashes and so on.
- [ ] Implement hardware supports for sdcard modules, spi flashes.
- [ ] Workarounds with C based libraries safetiness and reduce the `unsafe` blocks.
- [ ] Add webui implementations for `STA_STATION` mode to connect to wifi at runtime.
- [ ] Considering implementing webui for hardware configuring and VFS configuring with zero-cost at runtime?
- [ ] Using telegram bot as FUSE?
- [ ] Separate the project into crates for reusability for future projects?
- [ ] Tunning configs and reduce the runtime costs.
- [ ] Support encrypting.
- [ ] Support multiple boards and hardwares.
