# v0.2.0
## Bug Fixes
* channel info将不会再在列表中显示文件、隐藏目录这类与channel无关的内容。
* 补充apply中tag缺失的字段。
## Features
* 添加从目录结构或json结构读取文件名，并导入目录的功能。

# v0.1.1
## Bug Fixes
* source-data/connect增加对code为空的tag的异常处理。当tag的code/name/other_name皆空时忽略tag，否则抛出一个错误。
## Features
* API更新适配。新增setting.import.rules[].extras[].translateUnderscoreToSpace参数。