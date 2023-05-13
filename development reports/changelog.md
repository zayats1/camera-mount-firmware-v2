Camera mount firmware changelog:

## 0.0.1

## Key commits

- **commit:** 2f657200eb456fe59e8b627367af4c75043248e3
  
  initial commit

- **commit:** 11b42ebcfd581b5152ec5bb31dbef76e0b1555f0 
  
  added the servo driver I tested earlier

- **commit:** e6b7f93bfba80d084427877fe3abeba82c71a377
  
  I added prototype  to the stepper motor driver

- **commits:**  3989b00c7adccfa764bf647aaee57926fb6b7ca8 
  
  and  e6295511729d8e21278290ffb7d4b70cdb4104f2
  
  <u>My project is MPL licensed now</u>

and more smaller changes

### 0.0.2

1. Make stepper driver run non blocking  
   
   **commit:** 9e35165c42b29fea2273d63796456411896a04ed
   
   **move stepper to the second core for now**

2. Make a branch with some RTOS
   
   **branch:**  rtos
   
   **commit:** f2ae96c84523d2f684573fc4673bb855b0b8ffe8
   
   **this branch with rtos doesn't compile yet**
   
   I chose embassy because it looks more stable to me

and code cleanup

### 0.0.3

Uart comunication finnaly works as it should be, for now

merge branch **uart** into **main**

1. add proto data parser
   
   **commit:** 36b8ce46bd71de114d4587c5e7bc20bf32bc06b3
   
   **parse data prototype**

2. add UART comunication
   
   **commit:** 36b8ce46bd71de114d4587c5e7bc20bf32bc06b3
   
   **parse implementation and ditching dma**

## TODO

### 0.0.4

1. add some tests with defmt-test
2. merge timer async runtime

### 0.0.5

1. add some calculations and servo code to main
2. add some unit tests

### 0.0.6

1. add more calculations and test with bluetooth module
2. do some testing with real stepper motor and A4988 driver module

### 0.0.7

1. add more calculations 

2. cleanup code a bit

## <u>0.1.0</u>

1. test the project on my custom dev kit

2. small changes and refactor code

>  Bogdan Zayats -- your cute stargazer hare
