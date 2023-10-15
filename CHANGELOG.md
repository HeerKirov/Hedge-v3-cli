# v0.1.1
## Bug Fixes
* source-data/connect增加对code为空的tag的异常处理。当tag的code/name/other_name皆空时忽略tag，否则抛出一个错误。
## Features
* API更新适配。新增setting.import.rules[].extras[].translateUnderscoreToSpace参数。