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
