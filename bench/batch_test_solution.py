#!/usr/bin/env python3
"""
批量测试滑动验证码 /puzzle/solution 接口
功能：
1. 读取之前保存的测试数据（JSON 文件）
2. 提示用户输入滑动位置的 x 值
3. 发送验证请求
4. 更新 JSON 文件中的验证结果
"""

import requests
import json
import os
import argparse
from pathlib import Path
from typing import Dict, List, Optional
import glob


class SolutionTestClient:
    def __init__(self, base_url: str, data_dir: str = "test_results"):
        """
        初始化解决方案测试客户端
        
        Args:
            base_url: 服务器基础 URL
            data_dir: 测试数据目录（包含之前生成的 JSON 文件）
        """
        self.base_url = base_url.rstrip('/')
        self.data_dir = data_dir
        
        if not os.path.exists(data_dir):
            raise ValueError(f"数据目录不存在: {data_dir}")
    
    def verify_solution(self, puzzle_id: str, x: float) -> Dict:
        """
        验证解决方案
        
        Args:
            puzzle_id: 验证码 ID
            x: 滑块的 x 位置（归一化值 0.0-1.0）
            
        Returns:
            验证结果
        """
        url = f"{self.base_url}/puzzle/solution"
        payload = {
            "id": puzzle_id,
            "x": x
        }
        headers = {
            "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Accept": "application/json, text/plain, */*",
            "Accept-Language": "zh-CN,zh;q=0.9,en;q=0.8",
            "Content-Type": "application/json",
            "Connection": "keep-alive",
        }
        
        try:
            response = requests.post(url, json=payload, headers=headers, timeout=10)
            result = {
                "success": response.status_code == 200,
                "status_code": response.status_code,
                "response": response.json()
            }
            return result
        except requests.exceptions.RequestException as e:
            return {
                "success": False,
                "error": str(e),
                "status_code": getattr(e.response, 'status_code', None) if hasattr(e, 'response') else None
            }
    
    def load_test_data(self, json_file: str) -> Optional[Dict]:
        """
        加载测试数据 JSON 文件
        
        Args:
            json_file: JSON 文件路径
            
        Returns:
            测试数据字典
        """
        try:
            with open(json_file, 'r', encoding='utf-8') as f:
                return json.load(f)
        except Exception as e:
            print(f"❌ 读取文件失败: {json_file}, 错误: {e}")
            return None
    
    def save_test_data(self, json_file: str, data: Dict) -> bool:
        """
        保存测试数据
        
        Args:
            json_file: JSON 文件路径
            data: 要保存的数据
            
        Returns:
            是否保存成功
        """
        try:
            with open(json_file, 'w', encoding='utf-8') as f:
                json.dump(data, f, indent=2, ensure_ascii=False)
            return True
        except Exception as e:
            print(f"❌ 保存文件失败: {json_file}, 错误: {e}")
            return False
    
    def list_pending_tests(self) -> List[str]:
        """
        列出所有待测试的 JSON 文件
        
        Returns:
            JSON 文件路径列表
        """
        pattern = os.path.join(self.data_dir, "*_data.json")
        json_files = glob.glob(pattern)
        
        # 过滤出状态为 pending 的文件
        pending_files = []
        for json_file in json_files:
            data = self.load_test_data(json_file)
            if data and data.get('status') == 'pending':
                pending_files.append(json_file)
        
        return sorted(pending_files)
    
    def interactive_test(self, json_file: str) -> bool:
        """
        交互式测试单个验证码
        
        Args:
            json_file: JSON 文件路径
            
        Returns:
            是否成功完成测试
        """
        data = self.load_test_data(json_file)
        if not data:
            return False
        
        print(f"\n{'='*60}")
        print(f"📋 验证码信息:")
        print(f"   ID: {data['id']}")
        print(f"   Y 位置: {data['y']:.4f}")
        print(f"   尺寸: {data['width']}x{data['height']}")
        print(f"   拼图图片: {data['puzzle_file']}")
        print(f"   滑块图片: {data['piece_file']}")
        print(f"{'='*60}")
        
        # 提示用户输入 x 值
        try:
            x_input = input("请输入滑块的 x 位置 (0.0-1.0，或输入 's' 跳过): ").strip()
            
            if x_input.lower() == 's':
                print("⏭️  跳过此验证码")
                return True
            
            x = float(x_input)
            
            if x < 0.0 or x > 1.0:
                print("❌ x 值必须在 0.0 到 1.0 之间")
                return False
            
            # 发送验证请求
            print(f"🔄 正在验证... (ID: {data['id'][:8]}..., x: {x:.4f})")
            result = self.verify_solution(data['id'], x)
            
            # 更新数据
            data['status'] = 'tested'
            data['solution_result'] = {
                "x_submitted": x,
                "success": result['success'],
                "status_code": result.get('status_code'),
                "response": result.get('response'),
                "error": result.get('error')
            }
            
            # 保存更新后的数据
            self.save_test_data(json_file, data)
            
            # 显示结果
            if result['success']:
                print(f"✅ 验证成功！")
                print(f"   响应: {result['response']}")
            else:
                print(f"❌ 验证失败")
                print(f"   状态码: {result.get('status_code')}")
                print(f"   响应: {result.get('response', result.get('error'))}")
            
            return True
            
        except ValueError:
            print("❌ 输入的不是有效的数字")
            return False
        except KeyboardInterrupt:
            print("\n\n⏸️  测试中断")
            return False
    
    def auto_test(self, x_value: float) -> Dict:
        """
        自动测试所有待处理的验证码（使用固定的 x 值）
        
        Args:
            x_value: 要使用的 x 值
            
        Returns:
            测试结果统计
        """
        pending_files = self.list_pending_tests()
        
        if not pending_files:
            print("✨ 没有待测试的验证码")
            return {"total": 0, "success": 0, "failed": 0}
        
        print(f"\n🚀 开始自动测试")
        print(f"   找到 {len(pending_files)} 个待测试的验证码")
        print(f"   使用固定 x 值: {x_value:.4f}")
        print("-" * 60)
        
        results = {"total": 0, "success": 0, "failed": 0}
        
        for json_file in pending_files:
            data = self.load_test_data(json_file)
            if not data:
                continue
            
            results["total"] += 1
            
            # 发送验证请求
            result = self.verify_solution(data['id'], x_value)
            
            # 更新数据
            data['status'] = 'tested'
            data['solution_result'] = {
                "x_submitted": x_value,
                "success": result['success'],
                "status_code": result.get('status_code'),
                "response": result.get('response'),
                "error": result.get('error')
            }
            
            # 保存更新后的数据
            self.save_test_data(json_file, data)
            
            # 统计和显示
            if result['success']:
                results["success"] += 1
                print(f"✅ {os.path.basename(json_file)}: 验证成功")
            else:
                results["failed"] += 1
                print(f"❌ {os.path.basename(json_file)}: 验证失败 - {result.get('response', {}).get('error', 'Unknown')}")
        
        print("-" * 60)
        print(f"✨ 测试完成")
        print(f"   总数: {results['total']}")
        print(f"   成功: {results['success']}")
        print(f"   失败: {results['failed']}")
        print(f"   成功率: {results['success']/results['total']*100:.1f}%")
        
        return results
    
    def batch_interactive_test(self):
        """
        批量交互式测试
        """
        pending_files = self.list_pending_tests()
        
        if not pending_files:
            print("✨ 没有待测试的验证码")
            return
        
        print(f"\n🚀 找到 {len(pending_files)} 个待测试的验证码")
        print("提示: 输入 'q' 可以随时退出\n")
        
        for i, json_file in enumerate(pending_files, 1):
            print(f"\n[{i}/{len(pending_files)}] 测试: {os.path.basename(json_file)}")
            
            try:
                success = self.interactive_test(json_file)
                if not success:
                    break
            except KeyboardInterrupt:
                print("\n\n⏸️  测试中断")
                break
        
        print("\n✨ 测试会话结束")


def main():
    parser = argparse.ArgumentParser(
        description='批量测试滑动验证码 /puzzle/solution 接口',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
示例用法:
  # 交互式测试（逐个输入 x 值）
  python batch_test_solution.py -i

  # 自动测试（使用固定的 x 值）
  python batch_test_solution.py -x 0.5

  # 指定服务器和数据目录
  python batch_test_solution.py -H https://example.com -d ./test_results -i

  # 自动测试指定的单个文件
  python batch_test_solution.py -f test_results/20241016_120000_001_data.json -x 0.3
        """
    )
    
    parser.add_argument(
        '-H', '--host',
        default='http://localhost:8080',
        help='服务器地址 (默认: http://localhost:8080)'
    )
    
    parser.add_argument(
        '-d', '--data-dir',
        default='test_results',
        help='测试数据目录 (默认: test_results)'
    )
    
    parser.add_argument(
        '-i', '--interactive',
        action='store_true',
        help='交互式测试模式（逐个输入 x 值）'
    )
    
    parser.add_argument(
        '-x', '--x-value',
        type=float,
        help='自动测试模式：使用固定的 x 值 (0.0-1.0)'
    )
    
    parser.add_argument(
        '-f', '--file',
        help='测试指定的单个 JSON 文件'
    )
    
    args = parser.parse_args()
    
    # 创建客户端
    client = SolutionTestClient(
        base_url=args.host,
        data_dir=args.data_dir
    )
    
    # 根据参数执行不同的测试模式
    if args.file:
        # 单文件测试
        if args.x_value is not None:
            # 自动测试单个文件
            data = client.load_test_data(args.file)
            if data:
                result = client.verify_solution(data['id'], args.x_value)
                data['status'] = 'tested'
                data['solution_result'] = {
                    "x_submitted": args.x_value,
                    "success": result['success'],
                    "status_code": result.get('status_code'),
                    "response": result.get('response'),
                    "error": result.get('error')
                }
                client.save_test_data(args.file, data)
                
                if result['success']:
                    print(f"✅ 验证成功")
                else:
                    print(f"❌ 验证失败: {result}")
        else:
            # 交互式测试单个文件
            client.interactive_test(args.file)
    
    elif args.interactive:
        # 批量交互式测试
        client.batch_interactive_test()
    
    elif args.x_value is not None:
        # 批量自动测试
        if args.x_value < 0.0 or args.x_value > 1.0:
            print("❌ x 值必须在 0.0 到 1.0 之间")
            return
        client.auto_test(args.x_value)
    
    else:
        # 没有指定模式，显示帮助
        parser.print_help()
        print("\n💡 提示: 请使用 -i (交互式) 或 -x (自动测试) 参数")


if __name__ == '__main__':
    main()

