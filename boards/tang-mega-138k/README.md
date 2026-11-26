# Tang Mega 138K Dock 硬件参考手册

## 适用范围

本文描述以下硬件的器件标识、板载资源、引脚和约束：

- Sipeed Tang Mega 138K SOM 核心板。
- Sipeed Tang MEGA NEO Dock 138K 版底板。
- GW5AST-LV138PG484A FPGA。

本文以 31005 版 Dock 原理图为主要依据，并标明官方 CST 中记录的 31004、31002 UART 差异。内容不适用于采用 FPG676A 封装的 Tang Mega 138K Pro。

核心板集成 1 GiB DDR3 和 64/128 Mbit 外部配置 Flash。31005 Dock 电源树将 VCCIO2/3/4/5/10 接至 3.3 V，将 VCCIO6/7 接至 1.5 V。普通扩展 IO 使用 LVCMOS33 电平标准；DDR3 及 SOM 的 CPU_RESET 等部分信号使用 1.5 V Bank，不能按 3.3 V IO 使用。

## 术语

| 术语 | 含义 |
|---|---|
| CST | 引脚位置和电气属性约束文件 |
| SDC | 时序约束文件 |
| Bank 电压 | 一个 FPGA I/O Bank 的供电电压 |
| GCLK | 全局时钟输入引脚 |
| SOM | 通过 BTB 连接器安装到底板的核心板 |
| PMOD | 2x6 外设模组接口 |

## 工具链器件标识

不同工具使用不同的器件或板卡标识。

| 工具 | 标识 | 用途 |
|---|---|---|
| Gowin IDE | `GW5AST-LV138PG484A`，PBG484A，Device Version B/C | 按实物选择工程器件和版本 |
| nextpnr-himbaechel | `GW5AST-LV138PG484AC1/I0` | 选择布局布线目标器件 |
| gowin_pack | `GW5AST-138C` | 选择 C 版比特流封装器件 |
| openFPGALoader | `tangmega138k` | 选择配置目标板卡 |

Apicula 的 138K 示例在调用 `gowin_pack` 时还使用 `--cpu_as_gpio`。目前 Apicula 将该板标为 **Initial support**，仅支持基本时钟功能，不包含 HCLK、segments 等资源。开源流程以 C 版器件为目标，B 版实物的兼容性应单独验证。

Sipeed Wiki 还注明，Device Version B 的 SSRAM 支持受硅缺陷和核心电压条件限制，默认仅 C 版可用。需要 SSRAM 时，应按实物版本查阅 Wiki 的电压要求。

> **注意：** openFPGALoader 兼容表目前将本板列为 `GW5AST-138B (GW5AST-LV138FPG676A)`。其中 FPG676A 对应 138K Pro，138B 也与本文采用的 C 版开源流程不同。本板封装应为 PG484A，器件版本应按实物选择。

## 时钟

SOM 板载 50 MHz 晶振连接 FPGA 的 V22 引脚，电平标准为 LVCMOS33。

| 信号 | 引脚 | 说明 |
|---|---|---|
| `clk` | V22 | 50 MHz 晶振输入 |

50 MHz 时钟对应的 SDC 周期为 20 ns：

```tcl
create_clock -name clk_50m -period 20 [get_ports clk]
```

31005 版 Dock 的 MS5351 输出还连接 V13，原理图网络名为 `USR_CLK_IN`。该时钟由 MS5351 配置产生，不是固定的 50 MHz 输入；使用其他 Dock 版本时，应核对对应版本的原理图。

## LED

Dock 板载一个由 FPGA J16 驱动的 WS2812 RGB LED。Sipeed Wiki 中的 LED“4+8”指 4 个电量指示灯和 8 个 PMOD 外接 LED，不包含这个 WS2812。

| 位置 | 名称 | 引脚 | 说明 |
|---|---|---|---|
| SOM | READY | U12 | 默认用作 FPGA 配置状态灯 |
| SOM | DONE | G11 | FPGA 配置状态灯 |
| Dock | WS2812 | J16 | 板载 RGB LED；DOUT 还连接 J24 aRGB 灯带接口 |
| Dock | Battery-Indicator | 无 | 电源或电池电量指示，不能由普通用户 IO 驱动 |
| Dock | PMOD-LEDx8 | 见扩展连接器表 | 需外接模组，可由用户逻辑驱动 |

官方 PMOD LED 示例将 LED 寄存器取反后输出，因此示例中的点亮对应 FPGA 输出低电平。外接其他 LED 模组时，应自行确认极性。READY、DONE 的复用取决于 FPGA 配置和工具选项，本文不将它们作为普通用户 LED。

## 按键

Dock 提供三个用户按键和一个 reconfigure 按键。

| 按键 | 原理图位号 | 引脚 | 说明 |
|---|---|---|---|
| KEY.0 / USER_BUTTON0 | S4 | AA13 | 用户按键，网络名 `EX_KEY.0` |
| KEY.1 / USER_BUTTON1 | S2 | AB13 | 用户按键，网络名 `EX_KEY.1` |
| KEY.2 / USER_BUTTON2 | S1 | Y12 | 用户按键，网络名 `EX_KEY.2` |
| RECONFIG | SW7 | N12 | 专用 `BANK10_N12_RECONFIG`，不是普通用户 IO |

三个用户按键在 138K 上均使用 LVCMOS33。按键另一端接地，Dock 没有外部上拉，因此应配置 `PULL_MODE=UP`。空闲状态为高电平，按下状态为低电平。RECONFIG 在核心板上已有上拉。

> **注意：** SOM 的 F4 连接 CPU_RESET，官方 DDR 示例将其约束为 LVCMOS15、1.5 V。它不是上述 Dock 用户按键，也不能套用三个用户键的 3.3 V 约束。

## 调试串口

Dock 板载 BL616 调试器，提供 JTAG 和 UART，二者共用丝印为 DEBUG-USB2 的 USB-C 接口。使用 UART 时 JTAG 功能不可用；串口占用导致下载失败时，可重新插拔 USB 数据线。

| 方向 | 引脚 | 适用版本 | 说明 |
|---|---|---|---|
| FPGA 发送 | U15 | 官方 CST 未区分版本 | 连接调试器 UART 接收端 |
| FPGA 接收 | V14 | 31004 及以后 | 连接调试器 UART 发送端 |
| FPGA 接收 | Y14 | 31002 | 官方全引脚 CST 保留的旧版 FPGA UART RX 约束 |

上述 UART 信号均使用 LVCMOS33。SOM 还提供一个 JST SH1.0 8 Pin JTAG/UART 接口，供核心板脱离 Dock 调试使用。

如果 Gowin Programmer 可以下载而 openFPGALoader 不能工作，应检查并更新板载调试器固件。更新方法见 Sipeed Wiki。

## 扩展连接器

Dock 提供两个 PMOD 接口和两个 2x20 排针：

- J9 为 PMOD0，J8 为 PMOD1。两者的 1、2 脚为 3.3 V，3、4 脚为 GND。
- J13 靠近 PCIe 连接器，原理图标为 SDRAM0；J14 靠近 PMOD，原理图标为 SDRAM1。
- J14 与两个 PMOD 及 DVP 摄像头接口复用。使用这些接口前，应检查同一引脚是否已被其他功能占用。

下表列出两个 2x20 排针的 76 个可用 IO。每个排针的 11 脚为 5 V、12 脚为 GND，未列入表中。J13 的引脚位于 Bank 2，J14 的引脚位于 Bank 4，均使用 3.3 V VCCIO。

| 连接器 | 排针引脚 | FPGA 引脚 | 原理图网络名 | 复用功能 |
|---|---:|---|---|---|
| J13 | 1 | E14 | `SDRAM0_D0` | — |
| J13 | 2 | E13 | `SDRAM0_D1` | — |
| J13 | 3 | C15 | `SDRAM0_D2` | — |
| J13 | 4 | C14 | `SDRAM0_D3` | — |
| J13 | 5 | B13 | `SDRAM0_D4` | — |
| J13 | 6 | C13 | `SDRAM0_D5` | — |
| J13 | 7 | A14 | `SDRAM0_D6` | — |
| J13 | 8 | A13 | `SDRAM0_D7` | — |
| J13 | 9 | B16 | `SDRAM0_D15` | — |
| J13 | 10 | B15 | `SDRAM0_D14` | — |
| J13 | 13 | A16 | `SDRAM0_D13` | — |
| J13 | 14 | A15 | `SDRAM0_D12` | — |
| J13 | 15 | A20 | `SDRAM0_D11` | — |
| J13 | 16 | B20 | `SDRAM0_D10` | — |
| J13 | 17 | A21 | `SDRAM0_D9` | — |
| J13 | 18 | B21 | `SDRAM0_D8` | — |
| J13 | 19 | B18 | `SDRAM0_A12` | — |
| J13 | 20 | B17 | `SDRAM0_CLK` | — |
| J13 | 21 | C17 | `SDRAM0_A9` | — |
| J13 | 22 | D17 | `SDRAM0_A11` | — |
| J13 | 23 | A19 | `SDRAM0_A7` | — |
| J13 | 24 | A18 | `SDRAM0_A8` | — |
| J13 | 25 | D15 | `SDRAM0_A5` | — |
| J13 | 26 | D14 | `SDRAM0_A6` | — |
| J13 | 27 | D16 | `SDRAM0_~{WE}` | — |
| J13 | 28 | E16 | `SDRAM0_A4` | — |
| J13 | 29 | F14 | `SDRAM0_EXIO0` | 可通过 0 Ω 电阻连接 `SDRAM0_DM0` |
| J13 | 30 | F13 | `SDRAM0_EXIO1` | 可通过 0 Ω 电阻连接 `SDRAM0_DM1` |
| J13 | 31 | E17 | `SDRAM0_~{CAS}` | — |
| J13 | 32 | F16 | `SDRAM0_~{RAS}` | — |
| J13 | 33 | F19 | `SDRAM0_~{CS}` | — |
| J13 | 34 | F15 | `SDRAM0_BA0` | — |
| J13 | 35 | C20 | `SDRAM0_BA1` | — |
| J13 | 36 | D20 | `SDRAM0_A10` | — |
| J13 | 37 | D19 | `SDRAM0_A0` | — |
| J13 | 38 | E19 | `SDRAM0_A1` | — |
| J13 | 39 | C19 | `SDRAM0_A2` | — |
| J13 | 40 | C18 | `SDRAM0_A3` | — |
| J14 | 1 | R19 | `SDRAM1_D0` | `PMOD0_IO1`，J9.12 |
| J14 | 2 | P19 | `SDRAM1_D1` | `PMOD0_IO0`，J9.11 |
| J14 | 3 | U21 | `SDRAM1_D2` | `PMOD0_IO3`，J9.10 |
| J14 | 4 | T21 | `SDRAM1_D3` | `PMOD0_IO2`，J9.9 |
| J14 | 5 | R17 | `SDRAM1_D4` | `PMOD0_IO5`，J9.8 |
| J14 | 6 | P16 | `SDRAM1_D5` | `PMOD0_IO4`，J9.7 |
| J14 | 7 | T18 | `SDRAM1_D6` | `PMOD0_IO7`，J9.5 |
| J14 | 8 | R18 | `SDRAM1_D7` | `PMOD0_IO6`，J9.6 |
| J14 | 9 | W17 | `SDRAM1_D15` | — |
| J14 | 10 | V17 | `SDRAM1_D14` | — |
| J14 | 13 | W22 | `SDRAM1_D13` | — |
| J14 | 14 | W21 | `SDRAM1_D12` | — |
| J14 | 15 | P17 | `SDRAM1_D11` | — |
| J14 | 16 | N17 | `SDRAM1_D10` | — |
| J14 | 17 | N14 | `SDRAM1_D9` | — |
| J14 | 18 | N13 | `SDRAM1_D8` | — |
| J14 | 19 | V20 | `SDRAM1_A12` | — |
| J14 | 20 | U20 | `SDRAM1_CLK` | — |
| J14 | 21 | Y22 | `SDRAM1_A9` | `PMOD1_IO1`，J8.12；`CAM0_~{RST}` |
| J14 | 22 | Y21 | `SDRAM1_A11` | `PMOD1_IO0`，J8.11；`CAM0_VSYNC` |
| J14 | 23 | AB22 | `SDRAM1_A7` | `PMOD1_IO3`，J8.10；`CAM0_~{PWDN}` |
| J14 | 24 | AB21 | `SDRAM1_A8` | `PMOD1_IO2`，J8.9；`CAM0_HSYNC` |
| J14 | 25 | AA21 | `SDRAM1_A5` | `PMOD1_IO5`，J8.8；`CAM0_MCLK` |
| J14 | 26 | AA20 | `SDRAM1_A6` | `PMOD1_IO4`，J8.7；`CAM0_D7` |
| J14 | 27 | AB20 | `SDRAM1_~{WE}` | `PMOD1_IO7`，J8.6；`CAM0_PCLK` |
| J14 | 28 | AA19 | `SDRAM1_A4` | `PMOD1_IO6`，J8.5；`CAM0_D6` |
| J14 | 29 | AA18 | `SDRAM1_EXIO0` | 可通过 0 Ω 电阻连接 `SDRAM1_DM0` |
| J14 | 30 | AB18 | `SDRAM1_EXIO1` | 可通过 0 Ω 电阻连接 `SDRAM1_DM1` |
| J14 | 31 | Y19 | `SDRAM1_~{CAS}` | — |
| J14 | 32 | Y18 | `SDRAM1_~{RAS}` | — |
| J14 | 33 | T20 | `SDRAM1_~{CS}` | `CAM0_D5` |
| J14 | 34 | N15 | `SDRAM1_BA0` | `CAM0_D4` |
| J14 | 35 | U18 | `SDRAM1_BA1` | — |
| J14 | 36 | U17 | `SDRAM1_A10` | — |
| J14 | 37 | R16 | `SDRAM1_A0` | `CAM0_D2` |
| J14 | 38 | P15 | `SDRAM1_A1` | `CAM0_D1` |
| J14 | 39 | R14 | `SDRAM1_A2` | `CAM0_D3` |
| J14 | 40 | P14 | `SDRAM1_A3` | `CAM0_D0` |

> **注意：** 表中 `PMODx_IOy` 是原理图网络名，不一定等于示例程序中的 `led[y]` 数组索引。使用官方示例时，应以该示例附带的 CST 为准。

## 约束文件

### 引脚与电气属性

CST 文件使用 `IO_LOC` 指定引脚位置，使用 `IO_PORT` 指定电气属性。应为每个使用的引脚设置 `IO_LOC`，并按实际需要显式设置 `IO_PORT`。省略 `IO_PORT` 时，工具可能采用默认电气属性，不建议依赖默认值。

| 属性 | 常用取值 | 说明 |
|---|---|---|
| `IO_TYPE` | `LVCMOS33` | 普通 3.3 V IO 电平标准 |
| `DRIVE` | `8` | 常用输出驱动电流，单位 mA |
| `PULL_MODE` | `UP` / `DOWN` / `NONE` | 内部上拉、下拉或关闭上下拉 |
| `BANK_VCCIO` | `3.3` | 普通扩展 IO 的 Bank 电压，单位 V |

普通扩展 IO 应显式设置 `IO_TYPE=LVCMOS33` 和 `BANK_VCCIO=3.3`。按键输入还需要 `PULL_MODE=UP`。

DDR3 信号使用 `SSTL15` 或 `SSTL15D` 和 `BANK_VCCIO=1.5`，SOM 的 CPU_RESET 也位于 1.5 V Bank。CST 中的电压声明不会转换实际电平，不能将这些 1.5 V 引脚直接接到 3.3 V 外部信号，也不能把 DDR3 或 CPU_RESET 的约束复制到普通 GPIO。

## 配置存储

GW5AST 系列没有片内配置 Flash，非易失配置需要写入核心板的外部 Flash。

| 配置目标 | 特性 |
|---|---|
| SRAM | 配置立即生效，断电后丢失 |
| 外部 Flash | 配置在断电后保留 |

Gowin Programmer 可使用 `exFlash Erase,Program thru GAO-Bridge 5A` 模式（需要 1.9.11.03 或更新版本），或 `exFlash Erase,Program thru GAO-Bridge Arora V` 模式（需要 1.9.12 或更新版本）。Flash 型号选择 `Generic Flash`。

openFPGALoader 兼容表将 `tangmega138k` 的 SRAM 配置标为 OK，但板级 Flash 支持仍标为 TBD。使用开源工具写 Flash 前，应查阅所用版本的兼容说明并单独验证。

## 参考资料

- [Sipeed Wiki：Tang Mega 138K](https://wiki.sipeed.com/hardware/zh/tang/tang-mega-138k/mega-138k.html)
- [Tang MEGA NEO Dock 138K 31005 原理图](https://dl.sipeed.com/shareURL/TANG/Mega_138K_60K/02_Schematic/DOCK/Tang_Mega_NEO_Dock-138K_31005_Schematics.pdf)
- [Tang Mega 138K SOM 30354 原理图](https://dl.sipeed.com/shareURL/TANG/Mega_138K_60K/02_Schematic/SOM/tang_mega_138k_30354_Schematics..pdf)
- [Sipeed 官方全引脚 CST](https://dl.sipeed.com/shareURL/TANG/Mega_138K_60K/08_Misc/tang_mega_138K_pins.cst)
- [Sipeed 官方示例](https://github.com/sipeed/TangMega-138K-example)
- [Apicula 项目](https://github.com/YosysHQ/apicula)
- [Apicula 138K 构建示例](https://github.com/YosysHQ/apicula/blob/master/examples/gw5a/Makefile)
- [openFPGALoader 兼容板卡列表](https://trabucayre.github.io/openFPGALoader/compatibility/board.html)
- [Sipeed Wiki：常见问题](https://wiki.sipeed.com/hardware/zh/tang/common-doc/questions.html)
- [Sipeed Wiki：更新板载调试器](https://wiki.sipeed.com/hardware/zh/tang/common-doc/update_debugger.html)
