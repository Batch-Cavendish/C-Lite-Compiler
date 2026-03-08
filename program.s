.intel_syntax noprefix
.global main
.extern printInt

factorial:
  push rbp
  mov rbp, rsp
  sub rsp, 16
  mov [rbp -8], rdi
  mov rax, [rbp -8]
  push rax
  mov rax, 2
  mov rdi, rax
  pop rax
  cmp rax, rdi
  setle al
  movzx rax, al
  cmp rax, 0
  je .L1
  mov rax, [rbp -8]
  jmp .L0
  jmp .L2
.L1:
.L2:
  mov rax, [rbp -8]
  push rax
  mov rax, 1
  mov rdi, rax
  pop rax
  sub rax, rdi
  push rax
  pop rdi
  call factorial
  push rax
  mov rax, [rbp -8]
  push rax
  mov rax, 2
  mov rdi, rax
  pop rax
  sub rax, rdi
  push rax
  pop rdi
  sub rsp, 8
  call factorial
  add rsp, 8
  mov rdi, rax
  pop rax
  imul rax, rdi
  jmp .L0
.L0:
  mov rsp, rbp
  pop rbp
  ret

main:
  push rbp
  mov rbp, rsp
  mov rax, 5
  push rax
  pop rdi
  call factorial
  mov rdi, rax
  call printInt
  mov rax, 0
  jmp .L3
.L3:
  mov rsp, rbp
  pop rbp
  ret

