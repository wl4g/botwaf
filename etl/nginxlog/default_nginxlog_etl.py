# import os
# import gzip
# import re
# import csv
# from datetime import datetime
# from typing import Dict, List, Optional
# import ipaddress

# RAW_DIR = os.path.join(os.path.dirname(__file__), "raw")
# SAMPLE_DIR = os.path.join(os.path.dirname(__file__), "samples")
# CSV_HEADERS = [
#     "client_ip",
#     "req_headers",
#     "req_body",
#     "resp_headers",
#     "resp_body",
#     "resp_status",
#     "feature_marked",
# ]

# # Nginx logs parse patterns
# LOG_PATTERN = re.compile(
#     r'(?P<client_ip>\S+) - - \[(?P<timestamp>.*?)\] '
#     r'"(?P<method>\S+) (?P<path>\S+) \S+" '
#     r'(?P<resp_status>\d+) \d+ "(?P<referer>.*?)" "(?P<user_agent>.*?)"'
# )

# # 威胁检测规则（基于真实黑客特征）
# THREAT_RULES = [
#     {
#         "name": "Scanning_Attempt",
#         "conditions": [
#             {"field": "resp_status", "operator": "in", "value": ["404"]},
#             {"field": "path", "operator": "regex", "value": r"\b(php|backup|secret|password)\b"},
#             {"field": "method", "operator": "equals", "value": "HEAD"},
#             {"field": "user_agent", "operator": "contains", "value": "Custom-AsyncHttpClient"},
#         ],
#         "exclude_ips": ["14.19.0.0/16"],
#         "priority": 1,
#     },
#     {
#         "name": "SQL_Injection",
#         "conditions": [
#             {"field": "path", "operator": "regex", "value": r"(\bUNION\s+SELECT\b|\bDROP\s+TABLE\b|'\s*OR\s*1=1)"},
#         ],
#         "priority": 2,
#     },
#     {
#         "name": "XSS_Attack",
#         "conditions": [
#             {"field": "path", "operator": "regex", "value": r"(<script\b|onerror\s*=|javascript:)"},
#         ],
#         "priority": 3,
#     },
# ]

# def parse_log_line(line: str) -> Optional[Dict]:
#     """解析单行Nginx日志"""
#     match = LOG_PATTERN.match(line.strip())
#     if not match:
#         return None
#     return match.groupdict()

# def is_ip_excluded(ip: str, exclude_list: List[str]) -> bool:
#     """检查IP是否在排除列表"""
#     try:
#         ip_obj = ipaddress.ip_address(ip)
#         for network in exclude_list:
#             if ip_obj in ipaddress.ip_network(network):
#                 return True
#     except ValueError:
#         pass
#     return False

# def detect_threats(log_entry: Dict) -> str:
#     """威胁检测逻辑"""
#     threats = []
#     for rule in THREAT_RULES:
#         conditions_met = 0
#         # 条件检查
#         for condition in rule.get("conditions", []):
#             field = condition["field"]
#             operator = condition["operator"]
#             value = condition["value"]
            
#             # 字段获取（兼容Nginx日志字段）
#             entry_value = log_entry.get(field, "")
            
#             # 条件判断
#             if operator == "equals" and entry_value == value:
#                 conditions_met += 1
#             elif operator == "contains" and value in entry_value:
#                 conditions_met += 1
#             elif operator == "regex" and re.search(value, entry_value, re.IGNORECASE):
#                 conditions_met += 1
#             elif operator == "in" and entry_value in value:
#                 conditions_met += 1
        
#         # 排除可信IP
#         if is_ip_excluded(log_entry["client_ip"], rule.get("exclude_ips", [])):
#             continue
        
#         # 规则匹配（至少满足所有条件）
#         if conditions_met == len(rule.get("conditions", [])):
#             threats.append(rule["name"])
    
#     # 标记优先级（多个规则时取最高优先级）
#     if threats:
#         return "Yes"
#     return "No"

# def process_log_file(filepath: str) -> List[Dict]:
#     """处理单个日志文件"""
#     samples = []
#     with gzip.open(filepath, "rt") as f:
#         for line in f:
#             log_entry = parse_log_line(line.strip())
#             if not log_entry:
#                 continue
            
#             # 构建统一字段（兼容其他来源日志）
#             sample = {
#                 "client_ip": log_entry.get("client_ip", ""),
#                 "req_headers": f"method={log_entry.get('method', '')};path={log_entry.get('path', '')};"
#                               f"referer={log_entry.get('referer', '')};user_agent={log_entry.get('user_agent', '')}",
#                 "req_body": "",  # Nginx默认不记录请求体
#                 "resp_headers": "",  # Nginx默认不记录响应头
#                 "resp_body": "",  # Nginx默认不记录响应体
#                 "resp_status": log_entry.get("resp_status", ""),
#                 "feature_marked": detect_threats(log_entry),
#             }
#             samples.append(sample)
#     return samples

# def save_to_csv(samples: List[Dict], output_file: str):
#     """保存为CSV文件"""
#     os.makedirs(SAMPLE_DIR, exist_ok=True)
#     with open(output_file, "w", newline="") as f:
#         writer = csv.DictWriter(f, fieldnames=CSV_HEADERS)
#         writer.writeheader()
#         writer.writerows(samples)

# def main():
#     # 遍历处理所有.gz文件
#     for filename in os.listdir(RAW_DIR):
#         if not filename.endswith(".gz"):
#             continue
#         filepath = os.path.join(RAW_DIR, filename)
#         print(f"Processing {filepath}...")
#         samples = process_log_file(filepath)
#         # 按时间戳命名输出文件
#         timestamp = datetime.now().strftime("%Y%m%d%H%M%S")
#         output_file = os.path.join(SAMPLE_DIR, f"threat_logs_{timestamp}.csv")
#         save_to_csv(samples, output_file)
#         print(f"Saved {len(samples)} records to {output_file}")

# if __name__ == "__main__":
#     main()