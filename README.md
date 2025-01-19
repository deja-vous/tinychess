
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





