

=======
# tinychess
a very tiny chess engine I made in rust. Plays ~1800 ELO. 

# main.rs

At startup, our bot reads our authentication token and connects to the LICHESS event stream. It "listens" for incoming events such as challenges and game starts. When a game challenge arrives, the bot automatically accepts it.

For each new game, a local gametracker is created to keep track of the game state, current board position, bot's color and the game ID.

When it's the bots turn to move (when the board's side to move matches with our bots color AND the game is ongoing (not stopped or checkmated)), the bot calculates its best move (using our engine.rs file). Once we have figured out what move to play, it is formatted as UCI (Universal Chess Interface) and sent to Lichess. I also faced issues with rate limiting for some reason, so a small delay is introduced to tackle that issue.

# engine.rs

Contains logic for engine itself
I used the negamax function, Quiscence search, MVV-LVA, Iterative deepening for this. I also used PSTs as an added bonus. 

# psts.rs 

Contains piece-square-tables for each piece

# Installation 


# Installation

First ensure you have Rust installed on your system. 

Then, clone this repository:

``git clone https://github.com/hwenhwat/snakefish``

``cd snakefish``

Then, build the project.

``cargo build``

Then, run the engine

``cargo run``

The code is written in such a way that it will not run without you putting your Lichess Bot token in your Environment variables. Please run ``export LICHESS_BOT_TOKEN = {your_bot_token}`` 
Please also add the name of your bot in the code. I had issues with the local board and lichess board desyncing, so the bot's name needs to be added in the code to fix this. 

