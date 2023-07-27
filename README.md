# hadijatek

Hadijáték is a Diplomacy-like game invented at BME VeBio's FEB camp.

This project was started after playing a trial game with the old Haskell version,
which was full of bugs, and was hard to manage.
The webpage worked off an extremely basic CGI, and was quite slow.

## Components
### prelude
All common structs and methods.

### create_map
Take an Inkscape SVG, and create an initial game map.
Currently no Bezier curves are supported.

### webui
The webui: where players interact with the game.
Players are authenticated, submit orders, and see the state of the game.

### executor
Once all orders are submitted for a turn,
this tool executes all orders, resolves conflicts,
assigns retreats, etc.
In effect, it is a state transformer.

*THIS PROJECT IS STILL IN DEVELOPMENT*

The old Haskell repo:
[hadijatek](https://github.com/Atila-M-Schrieber/hadijatek_haskell)
