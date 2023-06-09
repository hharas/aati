/* بسم الله الرحمن الرحيم

   Aati - Cross-platform Package Manager written in Rust.
   Copyright (C) 2023  Husayn Haras

   This program is free software: you can redistribute it and/or modify
   it under the terms of version 3 of the GNU General Public License
   as published by the Free Software Foundation.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

pub const POSSIBLE_TARGETS: [&str; 130] = [
    "x86-linux",
    "x86-macos",
    "x86-ios",
    "x86-freebsd",
    "x86-dragonfly",
    "x86-netbsd",
    "x86-openbsd",
    "x86-solaris",
    "x86-android",
    "x86-windows",
    "x86_64-linux",
    "x86_64-macos",
    "x86_64-ios",
    "x86_64-freebsd",
    "x86_64-dragonfly",
    "x86_64-netbsd",
    "x86_64-openbsd",
    "x86_64-solaris",
    "x86_64-android",
    "x86_64-windows",
    "arm-linux",
    "arm-macos",
    "arm-ios",
    "arm-freebsd",
    "arm-dragonfly",
    "arm-netbsd",
    "arm-openbsd",
    "arm-solaris",
    "arm-android",
    "arm-windows",
    "aarch64-linux",
    "aarch64-macos",
    "aarch64-ios",
    "aarch64-freebsd",
    "aarch64-dragonfly",
    "aarch64-netbsd",
    "aarch64-openbsd",
    "aarch64-solaris",
    "aarch64-android",
    "aarch64-windows",
    "loongarch64-linux",
    "loongarch64-macos",
    "loongarch64-ios",
    "loongarch64-freebsd",
    "loongarch64-dragonfly",
    "loongarch64-netbsd",
    "loongarch64-openbsd",
    "loongarch64-solaris",
    "loongarch64-android",
    "loongarch64-windows",
    "m68k-linux",
    "m68k-macos",
    "m68k-ios",
    "m68k-freebsd",
    "m68k-dragonfly",
    "m68k-netbsd",
    "m68k-openbsd",
    "m68k-solaris",
    "m68k-android",
    "m68k-windows",
    "mips-linux",
    "mips-macos",
    "mips-ios",
    "mips-freebsd",
    "mips-dragonfly",
    "mips-netbsd",
    "mips-openbsd",
    "mips-solaris",
    "mips-android",
    "mips-windows",
    "mips64-linux",
    "mips64-macos",
    "mips64-ios",
    "mips64-freebsd",
    "mips64-dragonfly",
    "mips64-netbsd",
    "mips64-openbsd",
    "mips64-solaris",
    "mips64-android",
    "mips64-windows",
    "powerpc-linux",
    "powerpc-macos",
    "powerpc-ios",
    "powerpc-freebsd",
    "powerpc-dragonfly",
    "powerpc-netbsd",
    "powerpc-openbsd",
    "powerpc-solaris",
    "powerpc-android",
    "powerpc-windows",
    "powerpc64-linux",
    "powerpc64-macos",
    "powerpc64-ios",
    "powerpc64-freebsd",
    "powerpc64-dragonfly",
    "powerpc64-netbsd",
    "powerpc64-openbsd",
    "powerpc64-solaris",
    "powerpc64-android",
    "powerpc64-windows",
    "riscv64-linux",
    "riscv64-macos",
    "riscv64-ios",
    "riscv64-freebsd",
    "riscv64-dragonfly",
    "riscv64-netbsd",
    "riscv64-openbsd",
    "riscv64-solaris",
    "riscv64-android",
    "riscv64-windows",
    "s390x-linux",
    "s390x-macos",
    "s390x-ios",
    "s390x-freebsd",
    "s390x-dragonfly",
    "s390x-netbsd",
    "s390x-openbsd",
    "s390x-solaris",
    "s390x-android",
    "s390x-windows",
    "sparc64-linux",
    "sparc64-macos",
    "sparc64-ios",
    "sparc64-freebsd",
    "sparc64-dragonfly",
    "sparc64-netbsd",
    "sparc64-openbsd",
    "sparc64-solaris",
    "sparc64-android",
    "sparc64-windows",
];
