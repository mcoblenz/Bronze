BUILD_DIR = build

# Sorry, this isn't portable. We should probably transition to cmake or autoconf to fix this.
SYSROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX11.1.sdk/
OPTS=--target=x86_64-apple-macos -isysroot $(SYSROOT)


$(shell mkdir -p $(BUILD_DIR))

all: libbronze

$(BUILD_DIR)/Bronze.o: Bronze.c Bronze.h
	clang $(OPTS) -o $(BUILD_DIR)/Bronze.o -c Bronze.c

libbronze: $(BUILD_DIR)/Bronze.o
	ar rcu $(BUILD_DIR)/libbronze.a $+

clean:
	rm -rf $(BUILD_DIR)