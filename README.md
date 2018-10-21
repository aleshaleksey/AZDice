AZDice
======

A GUI tool for generating and visualising dice roll probability distributions.

Aims
----
Intended to help people trying to get game balance just right in homebrewed tabletop games.

Current State
-------------

***GUI***

Uses conrod for a simple GUI that which can be used to enter an input and display a graphical output.

***Supported Rolls***

Generates roll distributions over a full range of bonuses. Eg for an opposed 1d20 vs 1d20 roll, the full range of meaningful relative bonuses is -20 to +20.
Supports symmetrical roll distributions. (eg 3d6 vs 3d6 can be generates with "3d6", "3d6 vs" or "3d6 vs 3d6", 1d100 vs 1d100 can be generated with "100", "1d100", "d100", "1d100 vs 1d100")
Supports additive roll distributions. (eg 3d6+1d20 vs 3d6+1d20 can be generated with "3d6+1d20" or "3d6+1d20 vs 3d6+1d20")
Supports asymmetrical rolls. (eg 3d6 vs 1d20 can be generated by "3d6 vs 1d20")

***Output***

Creates a full .csv output file, for each unique distribution.
Displays the generated distribution as a graph.
Displays distribution in terminal window.

***Distribution generation algorithm***

Currently rolls virtual dice (using rand crate) lots of times. Then compares the answer.

Compiling
---------

Compiles on rust 1.28.0 with cargo build --release --features="winit glium libc"

Feedback
--------
Please let me know why this sucks and how it should be made less sucky.
