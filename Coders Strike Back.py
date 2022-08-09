import sys
import math

# Auto-generated code below aims at helping you parse
# the standard input according to the problem statement.

boost_used = False
# game loop
while True:
    # next_checkpoint_x: x position of the next check point
    # next_checkpoint_y: y position of the next check point
    # next_checkpoint_dist: distance to the next checkpoint
    # next_checkpoint_angle: angle between your pod orientation and the direction of the next checkpoint
    x, y, next_checkpoint_x, next_checkpoint_y, next_checkpoint_dist, next_checkpoint_angle = [int(i) for i in input().split()]
    opponent_x, opponent_y = [int(i) for i in input().split()]

    dist = next_checkpoint_dist
    angle = next_checkpoint_angle

    # Write an action using print
    # To debug: print("Debug messages...", file=sys.stderr)


    dist_mod = 20 * 2**-((dist-2000)/2000)
    thrust_ratio = (1 / (1 + 2**( (abs(angle) - 90 + dist_mod) / 2 )))

    print("angle: "+str(next_checkpoint_angle), file=sys.stderr)
    print("dist mod: "+str(dist_mod), file=sys.stderr)
    print("thrust ratio: "+str(thrust_ratio*100), file=sys.stderr)

    if next_checkpoint_dist > 2500 and abs(next_checkpoint_angle) < 10 and not boost_used:
        thrust = "BOOST"
        boost_used = True
    else:
        thrust = str(math.ceil(100 * thrust_ratio))

    print("thrust: "+thrust, file=sys.stderr)

    # You have to output the target position
    # followed by the power (0 <= thrust <= 100)
    # i.e.: "x y thrust"
    print("{} {} {}".format( next_checkpoint_x,
                             next_checkpoint_y,
                             thrust ) )
