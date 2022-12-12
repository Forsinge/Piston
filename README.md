# Piston

A revamped version of my old chess engine. The old version's playing strength was 
somewhere around 2000 ELO. The project is in early development and still has 
issues preventing from playing at that level, but its underlying speed is a lot 
higher.

Quick description of Piston's features:

Bitboards are used to represent the game, making use of lookup tables and
tricks like Hyperbola Quintessence to generate moves. For pure move generation, it 
should run at over 200 million (leaf) nodes per second on a good PC. To search and 
evaluate moves it uses the Principal Variation Search version of Negamax, though this 
part is likely what causes issues.

Piston runs using UCI protocol in order to interact with external clients and 
automate games. Even though it plays bad, it is capable of playing full games. 
Below is a list of currently supported commands:

<pre>
d                       display the current position
move [move]             make a move in the current position
pm                      print all legal moves in the current position
pt                      print all tactical moves in the current position
pq                      print all quiet moves in the current position
state                   print information about the engine state
stats                   print search statistics

position startpos       set the current position to the starting position
position fen [string]   set the current position to the given FEN string

stop                    terminate an ongoing search
go perft [depth]        search for the number of possible positions after [depth] moves
go [time constraint]    start a search for the best move given the time constraints
                        (currently not fully implemented)

uci                     used by clients
isready                 used by clients
exit                    exit
quit                    exit
</pre>

