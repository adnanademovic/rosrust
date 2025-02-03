^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
Changelog for package nav_msgs
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

1.13.1 (2021-01-11)
-------------------
* Update package maintainers (`#168 <https://github.com/ros/common_msgs/issues/168>`_)
* Add LoadMap service (`#164 <https://github.com/ros/common_msgs/issues/164>`_)
* Contributors: David V. Lu!!, Michel Hidalgo

1.13.0 (2020-05-21)
-------------------
* Bump CMake version to avoid CMP0048 warning (`#158 <https://github.com/ros/common_msgs/issues/158>`_)
* Contributors: Shane Loretz

1.12.7 (2018-11-06)
-------------------

1.12.6 (2018-05-03)
-------------------

1.12.5 (2016-09-30)
-------------------

1.12.4 (2016-02-22)
-------------------

1.12.3 (2015-04-20)
-------------------

1.12.2 (2015-03-21)
-------------------
* change type of initial_pose in SetMap service to PoseWithCovarianceStamped
* Contributors: Stephan Wirth

1.12.1 (2015-03-17)
-------------------
* updating outdated urls. fixes `#52 <https://github.com/ros/common_msgs/issues/52>`_.
* Adds a SetMap service message to support swap maps functionality in amcl
* Contributors: Tully Foote, liz-murphy

1.12.0 (2014-12-29)
-------------------

1.11.6 (2014-11-04)
-------------------

1.11.5 (2014-10-27)
-------------------

1.11.4 (2014-06-19)
-------------------

1.11.3 (2014-05-07)
-------------------
* Export architecture_independent flag in package.xml
* Contributors: Scott K Logan

1.11.2 (2014-04-24)
-------------------

1.11.1 (2014-04-16)
-------------------

1.11.0 (2014-03-04)
-------------------

1.10.6 (2014-02-27)
-------------------

1.10.5 (2014-02-25)
-------------------

1.10.4 (2014-02-18)
-------------------

1.10.3 (2014-01-07)
-------------------

1.10.2 (2013-08-19)
-------------------

1.10.1 (2013-08-16)
-------------------

1.10.0 (2013-07-13)
-------------------

1.9.16 (2013-05-21)
-------------------
* added action definition for getting maps
* update email in package.xml

1.9.15 (2013-03-08)
-------------------

1.9.14 (2013-01-19)
-------------------

1.9.13 (2013-01-13)
-------------------

1.9.12 (2013-01-02)
-------------------

1.9.11 (2012-12-17)
-------------------
* modified dep type of catkin

1.9.10 (2012-12-13)
-------------------
* add missing downstream depend
* switched from langs to message_* packages

1.9.9 (2012-11-22)
------------------

1.9.8 (2012-11-14)
------------------

1.9.7 (2012-10-30)
------------------
* fix catkin function order

1.9.6 (2012-10-18)
------------------
* updated cmake min version to 2.8.3, use cmake_parse_arguments instead of custom macro

1.9.5 (2012-09-28)
------------------

1.9.4 (2012-09-27 18:06)
------------------------

1.9.3 (2012-09-27 17:39)
------------------------
* cleanup
* cleaned up package.xml files
* updated to latest catkin
* fixed dependencies and more
* updated to latest catkin: created package.xmls, updated CmakeLists.txt

1.9.2 (2012-09-05)
------------------
* updated pkg-config in manifest.xml

1.9.1 (2012-09-04)
------------------
* use install destination variables, removed manual installation of manifests

1.9.0 (2012-08-29)
------------------
* updated to current catkin

1.8.13 (2012-07-26 18:34:15 +0000)
----------------------------------

1.8.8 (2012-06-12 22:36)
------------------------
* make find_package REQUIRED
* removed obsolete catkin tag from manifest files
* fixed package dependencies for several common messages (fixed `#3956 <https://github.com/ros/common_msgs/issues/3956>`_)
* adding manifest exports
* removed depend, added catkin
* stripping depend and export tags from common_msgs manifests as msg dependencies are now declared in cmake and stack.yaml.  Also removed bag migration exports
* common_msgs: removing migration rules as all are over a year old
* bye bye vestigial MSG_DIRS
* nav_msgs: getting rid of other build files and cleaning up
* common_msgs: starting catkin conversion
* adios rosbuild2 in manifest.xml
* catkin updates
* catkin_project
* Updated to work with new message generation macros
* More tweaking for standalone message generation
* Getting standalone message generation working... w/o munging rosbuild2
* more rosbuild2 hacking
* missing dependencies
* updating bagmigration exports
* rosbuild2 taking shape
* removing old exports ros`#2292 <https://github.com/ros/common_msgs/issues/2292>`_
* Added Ubuntu platform tags to manifest
* Adding a start pose to the GetPlan service
* Remove use of deprecated rosbuild macros
* Fixing migration rules for nav_msgs.
* Changed byte to int8, in response to map_server doc review
* changing review status
* adding documentation for `#2997 <https://github.com/ros/common_msgs/issues/2997>`_
* removing redundant range statements as per ticket:2997
* Adding documentation to the Odometry message to make things more clear
* manifest update
* updated description and url
* full migration rules
* adding child_frame_id as per discussion about odometry message
* Adding a header to Path
* Adding a header to the GridCells message
* Adding a new GridCells message for displaying obstacles in nav_view and rviz
* clearing API reviews for they've been through a bunch of them recently.
* fixing stack name
* Adding comments to path
* documenting messages
* Making odometry migration fail until we have worked out appropriate way to handle covariances.
* Changing naming of bag migration rules.
* Modifying migration rules for Odometry and WrenchStamped change of field names.
* Adding actual migration rules for all of the tested common_msgs migrations.
* `#2250 <https://github.com/ros/common_msgs/issues/2250>`_ getting rid of _with_covariance in Odometry fields
* nav_msgs: added missing srv export
* Adding migration rules to get migration tests to pass.
* removing last of robot_msgs and all dependencies on it
* moving Path from robot_msgs to nav_msgs `#2281 <https://github.com/ros/common_msgs/issues/2281>`_
* adding header to OccupancyGrid `#1906 <https://github.com/ros/common_msgs/issues/1906>`_
* First half of the change from deprecated_msgs::RobotBase2DOdom to nav_msgs::Odometry, I think all the c++ compiles, can't speak for functionality yet, also... the python has yet to be run... this may break some things
* moving PoseArray into geometry_msgs `#1907 <https://github.com/ros/common_msgs/issues/1907>`_
* fixing names
* Removing header since there's already one in the pose and fixing message definition to have variable names
* adding Odometry message as per API review and ticket:2250
* merging in the changes to messages see ros-users email.  THis is about half the common_msgs API changes
* Forgot to check in the services I added.... shoot
* Moving StaticMap.srv to GetMap.srv and moving Plan.srv to GetPlan.srv, also moving them to nav_msgs and removing the nav_srvs package
* Merging tha actionlib branch back into trunk
  r29135@att (orig r19792):  eitanme | 2009-07-27 18:30:30 -0700
  Creating a branch for actionlib.... hopefully for the last time
  r29137@att (orig r19794):  eitanme | 2009-07-27 18:32:49 -0700
  Changing ParticleCloud to PoseArray
  r29139@att (orig r19796):  eitanme | 2009-07-27 18:33:42 -0700
  Adding action definition to the rep
  r29148@att (orig r19805):  eitanme | 2009-07-27 18:47:39 -0700
  Some fixes... almost compiling
  r29165@att (orig r19822):  eitanme | 2009-07-27 20:41:07 -0700
  Macro version of the typedefs that compiles
  r29213@att (orig r19869):  eitanme | 2009-07-28 11:49:10 -0700
  Compling version of the ActionServer re-write complete with garbage collection, be default it will keep goals without goal handles for 5 seconds
  r29220@att (orig r19876):  eitanme | 2009-07-28 12:06:06 -0700
  Fix to make sure that transitions into preempting and recalling states actually happen
  r29254@att (orig r19888):  eitanme | 2009-07-28 13:27:40 -0700
  Forgot to actually call the cancel callback... addind a subscriber on the cancel topic
  r29267@att (orig r19901):  eitanme | 2009-07-28 14:41:09 -0700
  Adding text field to GoalStatus to allow users to throw some debugging information into the GoalStatus messages
  r29275@att (orig r19909):  eitanme | 2009-07-28 15:43:49 -0700
  Using tf remapping as I should've been doing for awhile
  r29277@att (orig r19911):  eitanme | 2009-07-28 15:48:48 -0700
  The navigation stack can now handle goals that aren't in the global frame. However, these goals will be transformed to the global frame at the time of reception, so for achieving them accurately the global frame of move_base should really be set to match the goals.
  r29299@att (orig r19933):  stuglaser | 2009-07-28 17:08:10 -0700
  Created genaction.py script to create the various messages that an action needs
  r29376@att (orig r20003):  vijaypradeep | 2009-07-29 02:45:24 -0700
  ActionClient is running. MoveBase ActionServer seems to be crashing
  r29409@att (orig r20033):  vijaypradeep | 2009-07-29 11:57:54 -0700
  Fixing bug with adding status trackers
  r29410@att (orig r20034):  vijaypradeep | 2009-07-29 11:58:18 -0700
  Changing from Release to Debug
  r29432@att (orig r20056):  vijaypradeep | 2009-07-29 14:07:30 -0700
  No longer building goal_manager_test.cpp
  r29472@att (orig r20090):  vijaypradeep | 2009-07-29 17:04:14 -0700
  Lots of Client-Side doxygen
  r29484@att (orig r20101):  vijaypradeep | 2009-07-29 18:35:01 -0700
  Adding to mainpage.dox
  r29487@att (orig r20104):  eitanme | 2009-07-29 18:55:06 -0700
  Removing file to help resolve merge I hope
  r29489@att (orig r20106):  eitanme | 2009-07-29 19:00:07 -0700
  Removing another file to try to resolve the branch
  r29492@att (orig r20108):  eitanme | 2009-07-29 19:14:25 -0700
  Again removing a file to get the merge working
  r29493@att (orig r20109):  eitanme | 2009-07-29 19:34:45 -0700
  Removing yet another file on which ssl negotiation fails
  r29500@att (orig r20116):  eitanme | 2009-07-29 19:54:18 -0700
  Fixing bug in genaction
* moving MapMetaData and OccGrid into nav_msgs `#1303 <https://github.com/ros/common_msgs/issues/1303>`_
* created nav_msgs and moved ParticleCloud there `#1300 <https://github.com/ros/common_msgs/issues/1300>`_
