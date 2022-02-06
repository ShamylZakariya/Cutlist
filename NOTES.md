# Notes:

- Easy optimization
    - sort once by number of boards needed and pick N with the lowest board count
    - sort that subset by score.
    This *should* resolve or reduce the issue where we have some N+1 board answers mixed in with N board answers

- Stack Generalization
    - Rename CutStack to RipStack
    - Rename Board to CrosscutStack
    - Create a Stack trait that embodies commonalities (dimensions, scoring, adding cuts?)
    - Make them implement stack trait

- We can do a secondary pass after we pick a layout which "cleans it up"
    - if we have small pieces like the apron mount, we can pack them in a secondary stack??
