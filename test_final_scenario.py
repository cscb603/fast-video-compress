import sys
import time
from pathlib import Path
from PyQt5.QtCore import QCoreApplication, QEventLoop
from video_compress.models import TranscodeRequest
from video_compress.worker import JobManager

def test_final_scenario():
    app = QCoreApplication(sys.argv)
    
    input_dir = Path("/Users/xtap/Downloads/御3 pro/DJI_001_STONE")
    # 由于 macOS 终端环境权限限制，Downloads 目录不可写
    # 我们改用项目目录下的 test_output 文件夹进行测试，证明核心逻辑是通的
    output_dir = "/Users/xtap/Documents/AI/test_output"
    
    # 创建输出目录
    try:
        Path(output_dir).mkdir(parents=True, exist_ok=True)
        print(f"✅ 输出目录已准备: {output_dir}")
    except Exception as e:
        print(f"❌ 无法创建输出目录: {e}")
        return
    
    # 挑选前 2 个 MP4 文件进行测试
    test_files = [str(f) for f in input_dir.glob("*.MP4")][:2]
    
    if not test_files:
        print("❌ 未在输入目录找到 MP4 文件")
        return

    print(f"--- 开始最终场景测试 ---")
    print(f"输入目录: {input_dir}")
    print(f"输出目录: {output_dir}")
    print(f"待处理文件: {test_files}")

    manager = JobManager(concurrency=1)
    
    completed_count = 0
    failed_count = 0
    total = len(test_files)
    
    loop = QEventLoop()

    def on_status(path, msg):
        print(f" [状态] {Path(path).name}: {msg}")
        if "完成" in msg:
            nonlocal completed_count
            completed_count += 1
            if completed_count + failed_count >= total:
                loop.quit()

    def on_failed(path, msg):
        print(f" ❌ [失败] {Path(path).name}: {msg}")
        nonlocal failed_count
        failed_count += 1
        if completed_count + failed_count >= total:
            loop.quit()

    def on_progress(path, val):
        if val % 20 == 0: # 减少打印频率
            print(f" ⏳ [进度] {Path(path).name}: {val}%")

    manager.itemStatus.connect(on_status)
    manager.itemProgress.connect(on_progress)
    # JobManager 没有直接暴露 failed 信号，但 Job 会触发 itemStatus 发送失败消息
    # 实际上 JobManager 的 _on_job_failed 会调用 itemStatus.emit(path, msg)
    
    reqs = [
        TranscodeRequest(
            input_path=p,
            output_dir=output_dir,
            pro=True,
            pro_encoder="h264",
            pro_height=720, # 为了测试快一点，用 720p
            bitrate_kbps=2000
        ) for p in test_files
    ]
    
    manager.enqueue(reqs)
    
    # 设置超时，防止死锁（例如 5 分钟）
    from PyQt5.QtCore import QTimer
    timer = QTimer()
    timer.setSingleShot(True)
    timer.timeout.connect(lambda: (print("❌ 测试超时"), loop.quit()))
    timer.start(300000) 
    
    loop.exec_()
    
    print(f"\n--- 测试结束 ---")
    print(f"成功: {completed_count}")
    print(f"失败: {failed_count}")
    
    # 检查输出文件是否存在
    for p in test_files:
        out_name = Path(p).stem + "_xiao.mp4"
        out_file = Path(output_dir) / out_name
        if out_file.exists():
            print(f"✅ 输出文件已生成: {out_file} ({out_file.stat().st_size / 1024 / 1024:.2f} MB)")
        else:
            print(f"❌ 输出文件未找到: {out_file}")

if __name__ == "__main__":
    test_final_scenario()
