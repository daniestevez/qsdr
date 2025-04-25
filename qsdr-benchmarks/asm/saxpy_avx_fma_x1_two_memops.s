        section .text
        global _start

_start:
        and rsp, 0xFFFFFFFFFFFFFFC0 ; align stack to 64 bytes
        sub rsp, 64 ; make room for 2x 32-byte buffers
        lea rbx, [rsp + 32] ; read buffer
        mov rcx, rsp ; write buffer
        xor rdx, rdx ; loop index (mock up)
        mov rax, 1000000000
        ;; zero initialize constants
        ;; (prevents working with nans and subnormals)
        vxorps ymm0, ymm0, ymm0
        vxorps ymm1, ymm1, ymm1
        ;; zero initialize read buffer
        ;; (prevents working with nans and subnormals)
        vmovaps [rbx], ymm0
.loop:
        vmovaps ymm2, [rbx + rdx]
        vfmadd132ps ymm2, ymm1, ymm0
        vmovaps [rcx + rdx], ymm2
        sub rax, 1
        jne .loop

        ;; exit
        mov rax, 60
        xor rdi, rdi
        syscall
