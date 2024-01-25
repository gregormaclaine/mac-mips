
.data     

output:     .asciiz  "Hello World"

.text



    main:

li $v0, 4
la $a0, output
syscall

li $v0, 10
syscall



