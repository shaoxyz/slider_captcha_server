#!/usr/bin/env python3
"""
批量测试滑动验证码 /puzzle 接口
功能：
1. 批量请求 /puzzle 接口
2. 保存返回的图片（puzzle 和 piece）为 png 或 jpg
3. 保存响应的 JSON 数据，方便后续测试 /puzzle/solution
"""

import requests
import base64
import json
import os
import argparse
import time
from datetime import datetime
from typing import Dict, List
from pathlib import Path


class PuzzleTestClient:
    def __init__(self, base_url: str, output_dir: str = "test_results", image_format: str = "png"):
        """
        初始化测试客户端
        
        Args:
            base_url: 服务器基础 URL，例如 http://localhost:8080
            output_dir: 输出目录
            image_format: 图片格式，png 或 jpg
        """
        self.base_url = base_url.rstrip('/')
        self.output_dir = output_dir
        self.image_format = image_format.lower()
        
        if self.image_format not in ['png', 'jpg', 'jpeg']:
            raise ValueError("图片格式只支持 png 或 jpg")
        
        # 创建输出目录
        Path(self.output_dir).mkdir(parents=True, exist_ok=True)
        
    def fetch_puzzle(self, width: int = 500, height: int = 300) -> Dict:
        """
        请求一个验证码
        
        Args:
            width: 图片宽度
            height: 图片高度
            
        Returns:
            包含响应数据的字典
        """
        url = f"{self.base_url}/puzzle"
        params = {"w": width, "h": height}
        headers = {
            "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Accept": "application/json, text/plain, */*",
            "Accept-Language": "zh-CN,zh;q=0.9,en;q=0.8",
            "Connection": "keep-alive",
        }
        
        try:
            # 增加重试机制
            max_retries = 3
            retry_delay = 1  # 秒
            
            for attempt in range(max_retries):
                try:
                    response = requests.get(url, params=params, headers=headers, timeout=30)
                    response.raise_for_status()
                    return {
                        "success": True,
                        "data": response.json(),
                        "status_code": response.status_code,
                        "width": width,
                        "height": height
                    }
                except (requests.exceptions.ConnectionError, requests.exceptions.Timeout) as retry_err:
                    if attempt < max_retries - 1:
                        print(f"   ⚠️  请求失败，{retry_delay}秒后重试... (尝试 {attempt + 1}/{max_retries})")
                        time.sleep(retry_delay)
                        retry_delay *= 2  # 指数退避
                    else:
                        raise retry_err
        except requests.exceptions.RequestException as e:
            return {
                "success": False,
                "error": str(e),
                "status_code": getattr(e.response, 'status_code', None) if hasattr(e, 'response') else None
            }
    
    def save_image(self, base64_data: str, filepath: str) -> bool:
        """
        保存 base64 编码的图片
        
        Args:
            base64_data: base64 编码的图片数据（可能包含 data:image/png;base64, 前缀）
            filepath: 保存路径
            
        Returns:
            是否保存成功
        """
        try:
            # 移除可能的 data URL 前缀
            if ',' in base64_data:
                base64_data = base64_data.split(',', 1)[1]
            
            # 解码 base64
            image_data = base64.b64decode(base64_data)
            
            # 写入文件
            with open(filepath, 'wb') as f:
                f.write(image_data)
            
            return True
        except Exception as e:
            print(f"❌ 保存图片失败: {filepath}, 错误: {e}")
            return False
    
    def save_test_result(self, result: Dict, index: int) -> Dict:
        """
        保存测试结果（图片和 JSON）
        
        Args:
            result: 从 fetch_puzzle 返回的结果
            index: 测试序号
            
        Returns:
            保存的文件信息
        """
        if not result['success']:
            print(f"❌ 测试 #{index} 失败: {result.get('error', 'Unknown error')}")
            return {
                "index": index,
                "success": False,
                "error": result.get('error')
            }
        
        # 生成时间戳
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        prefix = f"{timestamp}_{index:03d}"
        
        data = result['data']
        
        # 保存图片
        puzzle_file = os.path.join(self.output_dir, f"{prefix}_puzzle.{self.image_format}")
        piece_file = os.path.join(self.output_dir, f"{prefix}_piece.{self.image_format}")
        
        puzzle_saved = self.save_image(data['puzzle_image'], puzzle_file)
        piece_saved = self.save_image(data['piece_image'], piece_file)
        
        # 准备保存的 JSON 数据（用于后续验证测试）
        json_data = {
            "id": data['id'],
            "y": data['y'],
            "timestamp": timestamp,
            "width": result['width'],
            "height": result['height'],
            "puzzle_file": puzzle_file,
            "piece_file": piece_file,
            "status": "pending",  # 用于标记是否已经测试过 solution
            "solution_result": None  # 用于保存验证结果
        }
        
        # 保存 JSON
        json_file = os.path.join(self.output_dir, f"{prefix}_data.json")
        with open(json_file, 'w', encoding='utf-8') as f:
            json.dump(json_data, f, indent=2, ensure_ascii=False)
        
        file_info = {
            "index": index,
            "success": True,
            "id": data['id'],
            "puzzle_file": puzzle_file,
            "piece_file": piece_file,
            "json_file": json_file,
            "puzzle_saved": puzzle_saved,
            "piece_saved": piece_saved
        }
        
        # 打印结果
        status = "✅" if puzzle_saved and piece_saved else "⚠️"
        print(f"{status} 测试 #{index} - ID: {data['id'][:8]}... - 文件: {prefix}_*")
        
        return file_info
    
    def batch_test(self, count: int, width: int = 500, height: int = 300, delay: float = 0.5) -> List[Dict]:
        """
        批量测试
        
        Args:
            count: 测试次数
            width: 图片宽度
            height: 图片高度
            delay: 请求间隔（秒）
            
        Returns:
            所有测试结果的列表
        """
        print(f"\n🚀 开始批量测试")
        print(f"   服务器: {self.base_url}")
        print(f"   数量: {count}")
        print(f"   尺寸: {width}x{height}")
        print(f"   格式: {self.image_format}")
        print(f"   输出目录: {self.output_dir}")
        print(f"   请求间隔: {delay}秒")
        print("-" * 60)
        
        results = []
        
        for i in range(1, count + 1):
            # 请求验证码
            result = self.fetch_puzzle(width, height)
            
            # 保存结果
            file_info = self.save_test_result(result, i)
            results.append(file_info)
            
            # 添加延迟，避免服务器过载
            if i < count and delay > 0:
                time.sleep(delay)
        
        # 统计
        success_count = sum(1 for r in results if r.get('success', False))
        
        print("-" * 60)
        print(f"✨ 测试完成: {success_count}/{count} 成功")
        print(f"📁 结果保存在: {self.output_dir}")
        
        # 保存汇总文件
        summary_file = os.path.join(self.output_dir, "summary.json")
        summary = {
            "total_count": count,
            "success_count": success_count,
            "failure_count": count - success_count,
            "timestamp": datetime.now().isoformat(),
            "base_url": self.base_url,
            "width": width,
            "height": height,
            "image_format": self.image_format,
            "results": results
        }
        
        with open(summary_file, 'w', encoding='utf-8') as f:
            json.dump(summary, f, indent=2, ensure_ascii=False)
        
        print(f"📊 汇总文件: {summary_file}")
        
        return results


def main():
    parser = argparse.ArgumentParser(
        description='批量测试滑动验证码 /puzzle 接口',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
示例用法:
  # 本地测试，生成 10 个验证码
  python batch_test_puzzle.py -n 10

  # 指定服务器地址
  python batch_test_puzzle.py -H https://example.com -n 20

  # 自定义图片尺寸和格式
  python batch_test_puzzle.py -n 5 -w 600 -h 400 -f jpg

  # 指定输出目录
  python batch_test_puzzle.py -n 10 -o ./my_test_results
        """
    )
    
    parser.add_argument(
        '-H', '--host',
        default='http://101.126.148.100:8080',
        help='服务器地址 (默认: http://101.126.148.100:8080)'
    )
    
    parser.add_argument(
        '-n', '--count',
        type=int,
        default=50,
        help='生成验证码的数量 (默认: 10)'
    )
    
    parser.add_argument(
        '-w', '--width',
        type=int,
        default=500,
        help='图片宽度 (默认: 500)'
    )
    
    parser.add_argument(
        '-e', '--height',
        type=int,
        default=300,
        help='图片高度 (默认: 300)',
        dest='img_height'
    )
    
    parser.add_argument(
        '-f', '--format',
        choices=['png', 'jpg', 'jpeg'],
        default='png',
        help='图片格式 (默认: png)'
    )
    
    parser.add_argument(
        '-o', '--output',
        default='test_results',
        help='输出目录 (默认: test_results)'
    )
    
    parser.add_argument(
        '-d', '--delay',
        type=float,
        default=0.5,
        help='请求间隔时间（秒），避免服务器过载 (默认: 0.5)'
    )
    
    args = parser.parse_args()
    
    # 创建客户端并执行测试
    client = PuzzleTestClient(
        base_url=args.host,
        output_dir=args.output,
        image_format=args.format
    )
    
    client.batch_test(
        count=args.count,
        width=args.width,
        height=args.img_height,
        delay=args.delay
    )


if __name__ == '__main__':
    main()

