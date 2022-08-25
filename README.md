# asl_parser

Toy project to experiment with [ASL](https://alastairreid.github.io/specification_languages/). For now it just parses registers specification and pretty-prints format and values.

It requires regs.asl file generated from Arm's Machine Readable Architecture Specification which you can build using instructions from [alastairreid/mra_tools](https://github.com/alastairreid/mra_tools).
This file is searched in following places:

| OS | Path |
| - | - |
| Linux | $XDG_DATA_HOME/asl_parser/regs.asl or $HOME/.local/share/asl_parser/regs.asl |
| macOS |	$HOME/Library/Application Support/asl_parser/regs.asl |
| Windows |	{FOLDERID_RoamingAppData}\asl_parser\regs.asl |

Alternatively you can run `asl_parser init` or `cargo run -- init` to run build automatically. All dependencies for [alastairreid/mra_tools](https://github.com/alastairreid/mra_tools)  should be installed for it to work.

# Usage example
```
$ asl_parser 
Enter register names:
> cpsr
CPSR
+----+----+----+----+----+--------+------+-----+-----+----+--------+--------+---+---+---+---+------+------+
| 31 | 30 | 29 | 28 | 27 | 26..24 |  23  | 22  | 21  | 20 | 19..16 | 15..10 | 9 | 8 | 7 | 6 | 5..4 | 3..0 |
+----+----+----+----+----+--------+------+-----+-----+----+--------+--------+---+---+---+---+------+------+
| N  | Z  | C  | V  | Q  |        | SSBS | PAN | DIT |    |   GE   |        | E | A | I | F |      |  M   |
+----+----+----+----+----+--------+------+-----+-----+----+--------+--------+---+---+---+---+------+------+

CPSR> 0x123
CPSR
+----+----+----+----+----+--------+------+-----+-----+----+--------+--------+---+---+---+---+------+------+
| 31 | 30 | 29 | 28 | 27 | 26..24 |  23  | 22  | 21  | 20 | 19..16 | 15..10 | 9 | 8 | 7 | 6 | 5..4 | 3..0 |
+----+----+----+----+----+--------+------+-----+-----+----+--------+--------+---+---+---+---+------+------+
| N  | Z  | C  | V  | Q  |        | SSBS | PAN | DIT |    |   GE   |        | E | A | I | F |      |  M   |
+----+----+----+----+----+--------+------+-----+-----+----+--------+--------+---+---+---+---+------+------+
| 0  | 0  | 0  | 0  | 0  |   0    |  0   |  0  |  0  | 0  |   0    |   0    | 0 | 1 | 0 | 0 |  2   |  3   |
+----+----+----+----+----+--------+------+-----+-----+----+--------+--------+---+---+---+---+------+------+

CPSR> 
```
