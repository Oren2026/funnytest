def add(a, b):
    """計算兩個數字相加"""
    return a + b


# 用法說明
if __name__ == "__main__":
    # 基本用法
    result = add(3, 5)
    print(f"add(3, 5) = {result}")  # 輸出：8

    # 浮點數
    print(f"add(2.5, 1.3) = {add(2.5, 1.3)}")  # 輸出：3.8

    # 負數
    print(f"add(-10, 20) = {add(-10, 20)}")  # 輸出：10
