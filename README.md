# project-rs: imgui + opencv + sdl2

## development

### windows

Requirements:
- winget install Microsoft.VCRedist.2015+.x64 LLVM.LLVM
- choco install opencv

OpenCV Setup:
1. add environment vars:
    ```
    OPENCV_LINK_LIBS=opencv_world4100.lib
    OPENCV_LINK_PATHS=C:\tools\opencv\build\x64\vc16\lib
    OPENCV_INCLUDE_PATHS=C:\tools\opencv\build\include
    OPENCV_MSVC_CRT=static
    PATH=$PATH;C:\tools\opencv\build\x64\vc16\bin
    PATH=$PATH;C:\Program Files\LLVM\bin
    ```
1. open the developer shell:
    ```ps1
    git clone https://github.com/icedcoffeeee/project-rs
    cd project-rs
    cargo build
    ```

### ubuntu

Requirements:
- sudo apt install libopencv-dev

Run:
```sh
git clone https://github.com/icedcoffeeee/project-rs
cd project-rs
cargo build
```
