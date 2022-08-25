# asl_parser

Toy project to experiment with [ASL](https://alastairreid.github.io/specification_languages/).

It requires regs.asl file generated from Arm's Machine Readable Architecture Specification which you can build using instructions from [alastairreid/mra_tools](https://github.com/alastairreid/mra_tools)
This file is searched in following places:

| OS | Path |
| - | - |
| Linux | $XDG_DATA_HOME/asl_parser/regs.asl or $HOME/.local/share/asl_parser/regs.asl |
| macOS |	$HOME/Library/Application Support/asl_parser/regs.asl |
| Windows |	{FOLDERID_RoamingAppData}\asl_parser\regs.asl |

Alternatively you can run `asl_parser init` or `cargo run -- init` to run build automatically. All dependencies for [alastairreid/mra_tools](https://github.com/alastairreid/mra_tools)  should be installed for it to work.
