# Parameters example

Creates a node, creates parameters, prints them, and prints the whole parameter tree.

Comparing the results with `rosparam` is recommended.

You can pass a value for ~privbaz. Running
`cargo run _privbaz:="[1,2,3.3,'foo',"bar",[7,8,9],{x:5,y:2}]"`
will yield within the output:

```
(..)
Handling ~privbaz:
Get raw: Array([Int(1), Int(2), Double(3.3), String("foo"), String("bar"), Array([Int(7), Int(8), Int(9)]), Struct({"x": Int(5), "y": Int(2)})])
(..)
```

This is even more tolerant than `rospy`, which would require quotation marks around hash keys.
