uint8 CIRCLE=0
uint8 LINE_STRIP=1
uint8 LINE_LIST=2
uint8 POLYGON=3
uint8 POINTS=4

uint8 ADD=0
uint8 REMOVE=1

Header header
string ns	# namespace, used with id to form a unique id
int32 id          	# unique id within the namespace
int32 type        	# CIRCLE/LINE_STRIP/etc.
int32 action      	# ADD/REMOVE
geometry_msgs/Point position # 2D, in pixel-coords
float32 scale	 	# the diameter for a circle, etc.
std_msgs/ColorRGBA outline_color
uint8 filled	# whether to fill in the shape with color
std_msgs/ColorRGBA fill_color # color [0.0-1.0]
duration lifetime       # How long the object should last before being automatically deleted.  0 means forever


geometry_msgs/Point[] points # used for LINE_STRIP/LINE_LIST/POINTS/etc., 2D in pixel coords
std_msgs/ColorRGBA[] outline_colors # a color for each line, point, etc.
