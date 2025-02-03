^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
Changelog for package actionlib_msgs
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

1.13.1 (2021-01-11)
-------------------
* Update package maintainers (`#168 <https://github.com/ros/common_msgs/issues/168>`_)
* Contributors: Michel Hidalgo

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
* Fix spelling mistakes
* Contributors: trorornmn

1.12.4 (2016-02-22)
-------------------

1.12.3 (2015-04-20)
-------------------

1.12.2 (2015-03-21)
-------------------

1.12.1 (2015-03-17)
-------------------
* updating outdated urls. fixes `#52 <https://github.com/ros/common_msgs/issues/52>`_.
* Contributors: Tully Foote

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
* fix actionlib_msgs for dry
* use catkin_install_python() to install Python scripts `#29 <https://github.com/ros/common_msgs/issues/29>`_
* Contributors: Dirk Thomas

1.10.5 (2014-02-25)
-------------------
* removing usage of catkin function not guaranteed available fixes `#30 <https://github.com/ros/common_msgs/issues/30>`_
* Contributors: Tully Foote

1.10.4 (2014-02-18)
-------------------
* remove catkin usage from CMake extra file (fix `#27 <https://github.com/ros/common_msgs/issues/27>`_)
* Contributors: Dirk Thomas

1.10.3 (2014-01-07)
-------------------
* python 3 compatibility
* prefix invocation of python script with PYTHON_EXECUTABLE (`ros/genpy#23 <https://github.com/ros/genpy/issues/23>`_)
* make generated cmake relocatable

1.10.2 (2013-08-19)
-------------------

1.10.1 (2013-08-16)
-------------------

1.10.0 (2013-07-13)
-------------------

1.9.16 (2013-05-21)
-------------------
* update email in package.xml

1.9.15 (2013-03-08)
-------------------
* fix handling spaces in folder names (`ros/catkin#375 <https://github.com/ros/catkin/issues/375>`_)

1.9.14 (2013-01-19)
-------------------
* fix bug using ARGV in list(FIND) directly (`ros/genmsg#18 <https://github.com/ros/genmsg/issues/18>`_)

1.9.13 (2013-01-13)
-------------------

1.9.12 (2013-01-02)
-------------------

1.9.11 (2012-12-17)
-------------------
* add message_generation as a run dep for downstream
* modified dep type of catkin

1.9.10 (2012-12-13)
-------------------
* add missing downstream depend
* switched from langs to message_* packages

1.9.9 (2012-11-22)
------------------

1.9.8 (2012-11-14)
------------------
* add globbing for action files
* updated variable to latest catkin
* refactored extra file from 'in' to 'em'

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
* updated to latest catkin
* fixed dependencies and more
* updated to latest catkin: created package.xmls, updated CmakeLists.txt

1.9.2 (2012-09-05)
------------------
* updated pkg-config in manifest.xml
* updated catkin variables

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
* Fix up install-time finding of script, plus add a missing genmsg import
* Convert legacy rosbuild support to use newer genaction.py script
* Expose old actionlib_msgs interface to dry users.  Dry actionlib builds and
  tests cleanly.
* adding manifest exports
* removed depend, added catkin
* stripping depend and export tags from common_msgs manifests as msg dependencies are now declared in cmake and stack.yaml.  Also removed bag migration exports
* install a file that rosbuild users have hardcoded an include for
* bye bye vestigial MSG_DIRS
* rosbuild2 -> catkin
* no include dir in actionlib_msgs
* actionlib_msgs: getting rid of other build files
* adios rosbuild2 in manifest.xml
* catkin updates
* catkin_project
* catkin: only generate .msg files if .action file has changed
* catkin: changed actionlib_msg to generate .msg files at cmake time
* Integrate actionlib_msgs into catkin
* rosbuild2 on windows tweaks (more)
* rosbuild2 windows tweaks
* url fix
* removed extra slashes that caused trouble on OSX
* rosbuild2 taking shape
* rosbuild2 taking shape
* removing all the extra exports
* msg folder generation now parallel safe. `#4286 <https://github.com/ros/common_msgs/issues/4286>`_
* Fixing build dependency race condition. Trac `#4255 <https://github.com/ros/common_msgs/issues/4255>`_
* Added Ubuntu platform tags to manifest
* Now using /usr/bin/env python. Trac `#3863 <https://github.com/ros/common_msgs/issues/3863>`_
* Copying action generators from actionlib to actionlib_msgs
* updating review status
* Updating actionlib_msgs comments (`#3003 <https://github.com/ros/common_msgs/issues/3003>`_)
* filling out manifest
* Documenting GoalStatus message
* Forgot to commit files to actionlib_msgs
* Moving actionlib messages into common_msgs/actionlib_msgs. Trac `#2504 <https://github.com/ros/common_msgs/issues/2504>`_
