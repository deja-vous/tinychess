<<<<<<< HEAD

![snakefish](https://github.com/user-attachments/assets/e0ef265e-37ab-4e75-98ff-1b908e76a773)

# Snakefish

snakefish is a chess bot written in the rust programming language
                                                                                                                                                                                                     

Interfaces with lichess to make a bot that is playable. 
Refer to this blog to setup a bot:
https://lichess.org/@/thibault/blog/how-to-create-a-lichess-bot/FuKyvDuB

The engine uses fixed PST tables, which enhances evaluation by considering piece positions on the board. 

I have also employed the negamax algorithm with alpha-beta pruning. 

The bot has a customisable depth, but I've only tried it at 5 for testing purposes.

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

contains the Piece-Square-Tables for each piece. 
>>>>>>> f4abd81857299addafb32a2e9b4e88b6dfa560ab
