# Tang Primer 25K Dock 硬件参考手册

## 适用范围

本文描述以下硬件的器件标识、板载资源、引脚和约束：

- Sipeed Tang Primer 25K 核心板。
- Sipeed Tang Primer 25K Dock 底板。
- GW5A-LV25MG121 FPGA。

本板所有 FPGA I/O Bank 使用 3.3 V 电源。普通 IO 使用 LVCMOS33 电平标准。

## 术语

| 术语 | 含义 |
|---|---|
| CST | 引脚位置和电气属性约束文件 |
| SDC | 时序约束文件 |
| Bank 电压 | 一个 FPGA I/O Bank 的供电电压 |
| GCLK | 全局时钟输入引脚 |

## 工具链器件标识

不同工具使用不同的器件或板卡标识。

| 工具 | 标识 | 用途 |
|---|---|---|
| nextpnr-himbaechel | `GW5A-LV25MG121NC1/I0` | 选择布局布线目标器件 |
| gowin_pack | `GW5A-25A` | 选择比特流封装器件 |
| openFPGALoader | `tangprimer25k` | 选择配置目标板卡 |

## 时钟

板载 50 MHz 晶振连接 FPGA 的 E2 引脚，电平标准为 LVCMOS33。

| 信号 | 引脚 | 说明 |
|---|---|---|
| `clk` | E2 | 50 MHz 晶振输入 |

50 MHz 时钟对应的 SDC 周期为 20 ns：

```tcl
create_clock -name clk_50m -period 20 [get_ports clk]
```

## LED

Dock 底板没有独立的用户 LED。READY 状态灯可由用户逻辑驱动。

| 丝印 | 引脚 | 类型 | 用户逻辑可用 |
|---|---|---|---|
| READY | E8 | FPGA 状态灯 | 是 |
| DONE | D7 | FPGA 配置状态灯 | 否 |
| POWER | 无 | 电源指示灯 | 否 |

READY 为高电平点亮。DONE 由 FPGA 配置逻辑驱动，不能作为普通用户 IO。

> **注意：** L6 和 K6 分别连接 USB host 的 USB_P 和 USB_N。它们不连接 LED 或按键。

## 按键

Dock 底板提供两个用户按键。

| 丝印 | 引脚 | 说明 |
|---|---|---|
| S1 | H11 | 串联 100 Ω 电阻 |
| S2 | H10 | 串联 100 Ω 电阻 |

Sipeed 官方示例为按键输入配置 `PULL_MODE=DOWN`。在该配置下，空闲状态为低电平，按下状态为高电平。H10（S2）不具备 GCLK 功能，H11（S1）具备 GCLK 功能，两者也可以作为扩展 IO 使用。

## 调试串口

板载调试器提供 JTAG 和 UART，二者共用一个 USB-C 接口。

| 方向 | 引脚 | 说明 |
|---|---|---|
| FPGA 发送 | C3 | 连接调试器 UART 接收端 |
| FPGA 接收 | B3 | 连接调试器 UART 发送端 |

如果实际波特率恒为设定值的四倍，或输出持续乱码，应检查调试器固件版本。固件升级方法见 Sipeed Wiki。

## 扩展连接器

Dock 底板提供三个 PMOD 接口和一个 2x20 排针。排针兼容 Sipeed 40 Pin SDRAM 模组。

下表列出 64 个可用 IO。网络名带 `_40P` 后缀的引脚连接 2x20 排针。部分引脚具有时钟或按键复用功能。

| 索引 | 引脚 | 原理图网络名 | 连接器或复用功能 |
|---|---|---|---|
| 0 | G5 | G5_IOT72B | PMOD |
| 1 | F5 | F5_IOT72A | PMOD |
| 2 | G7 | G7_IOT68A | PMOD |
| 3 | G8 | G8_IOT68B | PMOD |
| 4 | H7 | H7_IOT66B | PMOD |
| 5 | H8 | H8_IOT66A | PMOD |
| 6 | L5 | L5_IOT63A | PMOD |
| 7 | K5 | K5_IOT63B | PMOD |
| 8 | J5 | J5_IOT61B | PMOD |
| 9 | H5 | H5_IOT61A | PMOD，GCLK |
| 10 | L9 | L9_IOT31A_40P | 2x20 排针 |
| 11 | K9 | K9_IOT31B_40P | 2x20 排针 |
| 12 | J8 | J8_IOT56A_40P | 2x20 排针，GCLK |
| 13 | K8 | K8_IOT56B_40P | 2x20 排针，GCLK |
| 14 | F6 | F6_IOT58B_40P | 2x20 排针 |
| 15 | F7 | F7_IOT58A_40P | 2x20 排针，GCLK |
| 16 | L2 | L2_IOR18A_40P | 2x20 排针 |
| 17 | L1 | L1_IOR18B_40P | 2x20 排针 |
| 18 | K1 | K1_IOR20A_40P | 2x20 排针 |
| 19 | K2 | K2_IOR20B_40P | 2x20 排针 |
| 20 | J4 | J4_IOR22A_40P | 2x20 排针 |
| 21 | K4 | K4_IOR22B_40P | 2x20 排针 |
| 22 | G2 | G2_IOR24A_40P | 2x20 排针 |
| 23 | G1 | G1_IOR24B_40P | 2x20 排针 |
| 24 | F1 | F1_IOB26B_40P | 2x20 排针 |
| 25 | B2 | B2_IOB4A_40P | 2x20 排针 |
| 26 | A1 | A1_IOB24A_40P | 2x20 排针 |
| 27 | C2 | C2_IOB4B_40P | 2x20 排针 |
| 28 | E1 | E1_IOB12B_40P | 2x20 排针 |
| 29 | F2 | F2_IOB26A_40P | 2x20 排针 |
| 30 | D1 | D1_IOB14B_40P | 2x20 排针 |
| 31 | E3 | E3_IOB60A_40P | 2x20 排针 |
| 32 | L4 | L4_IOR31A_40P | 2x20 排针 |
| 33 | J1 | J1_IOR33A_40P | 2x20 排针 |
| 34 | H1 | H1_IOB91A_40P | 2x20 排针 |
| 35 | G4 | G4_IOB89A_40P | 2x20 排针 |
| 36 | H2 | H2_IOB91B_40P | 2x20 排针 |
| 37 | H4 | H4_IOB89B_40P | 2x20 排针 |
| 38 | L3 | L3_IOR31B_40P | 2x20 排针 |
| 39 | J2 | J2_IOR33B_40P | 2x20 排针 |
| 40 | D11 | D11_IOL9A | PMOD |
| 41 | D10 | D10_IOL9B | PMOD |
| 42 | C10 | C10_IOL5B | PMOD |
| 43 | C11 | C11_IOL5A | PMOD |
| 44 | B11 | B11_IOL12A | PMOD |
| 45 | B10 | B10_IOL12B | PMOD |
| 46 | A10 | A10_IOL14B | PMOD |
| 47 | A11 | A11_IOL14A | PMOD |
| 48 | G10 | G10_IOT7B | PMOD |
| 49 | G11 | G11_IOT7A | PMOD |
| 50 | H10 | H10_IOT3B_S2 | 与按键 S2 复用 |
| 51 | H11 | H11_IOT3A_S1 | 与按键 S1 复用 |
| 52 | J10 | J10_IOT1B_40P | 2x20 排针 |
| 53 | J11 | J11_IOT1A_40P | 2x20 排针 |
| 54 | E11 | E11_IOL3A | PMOD |
| 55 | E10 | E10_IOL3B | PMOD |
| 56 | L11 | L11_IOT11B | PMOD |
| 57 | K11 | K11_IOT11A | PMOD |
| 58 | L10 | L10_IOT15A_40P | 2x20 排针 |
| 59 | K10 | K10_IOT15B_40P | 2x20 排针 |
| 60 | L8 | L8_IOT19B_40P | 2x20 排针 |
| 61 | L7 | L7_IOT19A_40P | 2x20 排针 |
| 62 | K7 | K7_IOT21A_40P | 2x20 排针 |
| 63 | J7 | J7_IOT21B_40P | 2x20 排针 |

> **注意：** 表中 PMOD 引脚在三个接口之间的具体分配尚未逐一确认。连接 PMOD 模组前，应核对底板丝印和原理图。

扩展连接器中的 GCLK 引脚包括 H5、J8、K8、F7 和 H11。外部时钟输入应优先使用 GCLK 引脚。

## 约束文件

### 引脚与电气属性

CST 文件使用 `IO_LOC` 指定引脚位置，使用 `IO_PORT` 指定电气属性。每个使用的引脚都需要这两类约束。

| 属性 | 常用取值 | 说明 |
|---|---|---|
| `IO_TYPE` | `LVCMOS33` | 3.3 V 电平标准 |
| `DRIVE` | `8` | 驱动电流，单位 mA |
| `PULL_MODE` | `UP` / `DOWN` / `NONE` | 内部上拉或下拉 |
| `BANK_VCCIO` | `3.3` | Bank 电压，单位 V |

每个 `IO_PORT` 都应显式设置 `IO_TYPE=LVCMOS33`。开源工具链可能使用 LVCMOS12 作为默认值。省略该属性可能导致 Bank 电压冲突。

## 配置存储

| 配置目标 | 特性 |
|---|---|
| SRAM | 配置立即生效，断电后丢失 |
| Flash | 配置在断电后保留 |

Flash 支持情况取决于配置工具及其版本。使用前应查阅工具的兼容说明。Gowin Programmer 可使用 exFlash 配置模式。

## 参考资料

- [Sipeed Wiki：Tang Primer 25K](https://wiki.sipeed.com/hardware/en/tang/tang-primer-25k/primer-25k.html)
- [Dock 底板原理图](https://dl.sipeed.com/shareURL/TANG/Primer_25K/02_Schematic)
- [Sipeed 官方示例](https://github.com/sipeed/TangPrimer-25K-example)
- [Apicula 项目](https://github.com/YosysHQ/apicula)
- [openFPGALoader 兼容板卡列表](https://trabucayre.github.io/openFPGALoader/compatibility/board.html)
