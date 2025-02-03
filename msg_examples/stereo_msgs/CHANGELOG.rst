^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
Changelog for package stereo_msgs
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

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
* fixed missing find genmsg

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

1.9.1 (2012-09-04)
------------------
* use install destination variables, removed manual installation of manifests

1.9.0 (2012-08-29)
------------------

1.8.13 (2012-07-26 18:34:15 +0000)
----------------------------------

1.8.8 (2012-06-12 22:36)
------------------------
* removed obsolete catkin tag from manifest files
* fixed package dependencies for several common messages (fixed `#3956 <https://github.com/ros/common_msgs/issues/3956>`_)
* adding manifest exports
* removed depend, added catkin
* stripping depend and export tags from common_msgs manifests as msg dependencies are now declared in cmake and stack.yaml.  Also removed bag migration exports
* common_msgs: removing migration rules as all are over a year old
* bye bye vestigial MSG_DIRS
* stereo_msgs: catkin'd
* adios rosbuild2 in manifest.xml
* missing dependencies
* updating bagmigration exports
* rosbuild2 taking shape
* removing old exports for msg/cpp and reving to 1.3.7 in preparation for release
* stereo_msgs into common_msgs `#4637 <https://github.com/ros/common_msgs/issues/4637>`_
