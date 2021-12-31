Next Steps:

## Rendering:

## Mouse Controls:
- scroll wheel zoom needs to center about mouse cursor
-

## Future:
- Shuffle algorithm is fine. Need to do a "smart" packing algorithm which is similar to the sorted approach, but "groups" cuts by their type and attempts to pack them better.
- We can do a secondary pass after we pick a layout which "cleans it up"
    - if we have small pieces like the apron mount, we can pack them in a secondary stack??
    - this can be generalized by making `CrosscutStack` and `RipStack` where `Board` becomes `CrosscutStack` and `CutStack` becomes `RipStack`
q