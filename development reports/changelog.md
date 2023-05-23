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

### 0.1.0

**The biggest pre release ever**

1. add  servo code to main
   **commit:**  fabe11b38e41269d9e3b7c50b88db37928e5fbc8

2. add some unit tests
   **commit:**  c0ed866693b550db6938021d98249f7e285c660b
   
   **unit test prototype**

3. make stepper motor run with timer
   **commit:** 9c01616e2ef10a9dd2b177052da8e6dff31291f8
   
   **Merge timer**

4. improve dataparsing algorithm and uart
   **commit:** 13b1c5029f9a8c1f717a0ef5789f00839dbbd9d4
   
   **improve parsing**
   
   **commit:** cdfa8ec8a1b3524d946923c37e2929bfa84c3c0c
   
   **Incomplete data should not be parsed**
   
   **commit:** def9c8c2891f2dc2a76970a1c74adaae1a710164
   
   **Merge pull request rom zayats1/queue**
   
   and bunch of small changes and cleanups

### 0.1.1

1. add more unit tests

### 0.1.2

1. do some testing with real stepper motor and A4988 driver module

2. test with bluetooth module

### 0.1.3

1. add some tests calculations 

2. cleanup code a bit

## 0.1.4

1. test the project on my custom dev kit

2. small changes and refactor code

>  Bogdan Zayats -- your cute stargazer hare
