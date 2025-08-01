# 搜索接口文档

## 概述

搜索接口提供高级插件搜索功能，支持全文搜索、标签过滤、排序、搜索建议等特性。采用灵活的查询语法，满足用户多样化的搜索需求。

## 基础URL

```
生产环境: https://api.plugins.geektools.com/api/v1
开发环境: http://localhost:3000/api/v1
```

## 接口详情

### 1. 高级搜索

提供功能丰富的插件搜索，支持复杂查询条件。

**接口地址**: `POST /search/advanced`

**请求头**:
```
Content-Type: application/json
Authorization: Bearer {token}  // 可选，用于个性化搜索
```

**请求参数**:
```json
{
  "query": "system monitor",           // 搜索关键词，可选
  "filters": {                        // 过滤条件，可选
    "tags": ["system", "monitoring"], // 标签过滤
    "author": "GeekTools Team",       // 作者过滤
    "license": "MIT",                 // 许可证过滤
    "status": "active",               // 状态过滤
    "rating": {                       // 评分过滤
      "min": 4.0,                     // 最低评分
      "max": 5.0                      // 最高评分
    },
    "downloads": {                    // 下载量过滤
      "min": 100,                     // 最少下载量
      "max": 10000                    // 最多下载量
    },
    "created_date": {                 // 创建时间过滤
      "from": "2024-01-01",           // 开始日期
      "to": "2024-12-31"              // 结束日期
    },
    "geektools_version": ">=1.0.0"    // GeekTools版本兼容性
  },
  "sort": {                          // 排序设置，可选
    "field": "downloads",             // 排序字段
    "order": "desc"                   // 排序方向
  },
  "pagination": {                    // 分页设置，可选
    "page": 1,                       // 页码，默认1
    "limit": 20                      // 每页数量，默认20，最大100
  },
  "highlight": true,                 // 是否高亮搜索关键词，可选
  "include_stats": true             // 是否包含统计信息，可选
}
```

**搜索字段说明**:

| 字段类型 | 支持字段 | 搜索方式 | 权重 |
|---------|----------|----------|------|
| **文本搜索** | name, description, author | 全文搜索 + 模糊匹配 | 高 |
| **精确匹配** | id, license, status | 精确匹配 | 中 |
| **标签搜索** | tags | 数组包含匹配 | 中 |
| **数值范围** | rating, downloads | 范围查询 | 低 |
| **时间范围** | created_at, updated_at | 时间区间查询 | 低 |

**排序字段选项**:
- `relevance`: 相关度排序（默认，仅当有搜索关键词时）
- `downloads`: 下载量排序
- `rating`: 评分排序  
- `created_at`: 创建时间排序
- `updated_at`: 更新时间排序
- `name`: 名称字母排序

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "plugins": [
      {
        "id": "system_monitor_demo",
        "name": "<em>System</em> <em>Monitor</em> Demo",  // 高亮显示
        "description": "A comprehensive <em>system</em> <em>monitoring</em> plugin...",
        "author": "GeekTools Team",
        "current_version": "1.2.0",
        "downloads": 1250,
        "rating": 4.5,
        "status": "active",
        "created_at": "2024-01-10T08:30:00Z",
        "updated_at": "2024-01-15T14:20:00Z",
        "tags": ["system", "monitoring", "demo"],
        "license": "MIT",
        "relevance_score": 0.85,           // 相关度分数
        "match_reasons": [                 // 匹配原因
          "title_match",
          "description_match",
          "tag_match"
        ]
      }
      // ... 更多插件
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 156,
      "pages": 8
    },
    "search_stats": {                    // 搜索统计（当include_stats=true时）
      "total_results": 156,
      "search_time_ms": 45,
      "filters_applied": 3,
      "suggestions": [
        "system monitoring",
        "system tools",
        "performance monitor"
      ]
    },
    "facets": {                         // 搜索结果分面统计
      "tags": {
        "system": 45,
        "monitoring": 32,
        "utility": 28,
        "performance": 19
      },
      "authors": {
        "GeekTools Team": 12,
        "Community Dev": 8,
        "Plugin Master": 5
      },
      "ratings": {
        "5.0": 23,
        "4.0-4.9": 89,
        "3.0-3.9": 35,
        "2.0-2.9": 8,
        "1.0-1.9": 1
      }
    }
  }
}
```

❌ **参数验证失败** (422 Unprocessable Entity):
```json
{
  "success": false,
  "error": "Validation failed",
  "details": {
    "pagination.limit": ["Limit cannot exceed 100"],
    "filters.rating.min": ["Minimum rating must be between 1 and 5"]
  }
}
```

### 2. 搜索建议

提供实时搜索建议和自动补全功能。

**接口地址**: `GET /search/suggestions`

**查询参数**:
| 参数名 | 类型 | 必需 | 说明 |
|--------|------|------|------|
| q | string | 是 | 搜索查询词，最少2个字符 |
| limit | integer | 否 | 建议数量，默认10，最大20 |
| type | string | 否 | 建议类型：all/plugins/authors/tags |

**请求示例**:
```bash
curl -X GET "http://localhost:3000/api/v1/search/suggestions?q=sys&limit=5&type=all"
```

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "suggestions": [
      {
        "text": "system monitor",
        "type": "plugin",
        "category": "popular_search",
        "match_count": 12,
        "highlight": "<em>sys</em>tem monitor"
      },
      {
        "text": "system tools",
        "type": "plugin", 
        "category": "plugin_name",
        "match_count": 8,
        "highlight": "<em>sys</em>tem tools"
      },
      {
        "text": "GeekTools System Team",
        "type": "author",
        "category": "author_name", 
        "match_count": 5,
        "highlight": "GeekTools <em>Sys</em>tem Team"
      },
      {
        "text": "system",
        "type": "tag",
        "category": "popular_tag",
        "match_count": 45,
        "highlight": "<em>sys</em>tem"
      }
    ],
    "query_completion": [              // 查询补全建议
      "system monitor",
      "system performance", 
      "system utilities"
    ],
    "popular_searches": [             // 热门搜索
      "system monitor",
      "file manager",
      "network tools"
    ]
  }
}
```

**建议类型说明**:
- `plugin`: 插件名称建议
- `author`: 作者名称建议  
- `tag`: 标签建议
- `category`: 分类建议

**建议来源**:
- `plugin_name`: 来自插件名称
- `plugin_description`: 来自插件描述
- `author_name`: 来自作者名称
- `popular_tag`: 热门标签
- `popular_search`: 热门搜索词
- `user_history`: 用户搜索历史（需要认证）

### 3. 热门搜索

获取当前热门的搜索关键词和趋势。

**接口地址**: `GET /search/trending`

**查询参数**:
| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| period | string | 否 | day | 统计周期：hour/day/week/month |
| limit | integer | 否 | 10 | 返回数量，最大50 |

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "trending_searches": [
      {
        "query": "system monitor",
        "search_count": 1245,
        "growth_rate": 15.5,           // 增长率(%)
        "rank": 1,
        "rank_change": 0               // 排名变化
      },
      {
        "query": "file manager", 
        "search_count": 987,
        "growth_rate": 8.2,
        "rank": 2,
        "rank_change": 1               // 上升1位
      },
      {
        "query": "network tools",
        "search_count": 756,
        "growth_rate": -2.1,           // 负数表示下降
        "rank": 3,
        "rank_change": -1              // 下降1位
      }
    ],
    "trending_tags": [
      {
        "tag": "system",
        "usage_count": 45,
        "growth_rate": 12.3,
        "related_searches": ["system monitor", "system tools"]
      },
      {
        "tag": "utility",
        "usage_count": 38,
        "growth_rate": 9.7,
        "related_searches": ["file utility", "system utility"]
      }
    ],
    "search_statistics": {
      "total_searches_today": 5678,
      "unique_queries_today": 1234,
      "average_results_per_search": 15.6,
      "most_active_hour": "14:00-15:00"
    }
  }
}
```

### 4. 搜索历史 (需要认证)

获取用户的个人搜索历史记录。

**接口地址**: `GET /search/history`

**请求头**:
```
Authorization: Bearer {token}  // 必需
```

**查询参数**:
| 参数名 | 类型 | 必需 | 默认值 | 说明 |
|--------|------|------|--------|------|
| limit | integer | 否 | 20 | 返回数量，最大100 |
| days | integer | 否 | 30 | 历史天数，最大90 |

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "data": {
    "search_history": [
      {
        "id": 1001,
        "query": "system monitor",
        "filters": {
          "tags": ["system"],
          "rating": {"min": 4.0}
        },
        "results_count": 12,
        "searched_at": "2024-01-15T10:30:00Z"
      },
      {
        "id": 1000,
        "query": "file manager",
        "filters": {},
        "results_count": 25,
        "searched_at": "2024-01-15T09:15:00Z"
      }
    ],
    "frequent_queries": [             // 常用查询
      {
        "query": "system monitor",
        "count": 5,
        "last_searched": "2024-01-15T10:30:00Z"
      }
    ],
    "statistics": {
      "total_searches": 45,
      "unique_queries": 23,
      "most_searched": "system monitor",
      "average_results": 18.2
    }
  }
}
```

### 5. 清除搜索历史 (需要认证)

清除用户的搜索历史记录。

**接口地址**: `DELETE /search/history`

**请求头**:
```
Authorization: Bearer {token}  // 必需
```

**请求参数** (可选):
```json
{
  "before_date": "2024-01-01",     // 清除指定日期前的记录
  "query_ids": [1001, 1002]       // 清除指定ID的记录
}
```

不提供参数时清除所有历史记录。

**响应示例**:

✅ **成功响应** (200 OK):
```json
{
  "success": true,
  "message": "Search history cleared successfully",
  "data": {
    "cleared_count": 45
  }
}
```

## 搜索语法

### 基础语法

| 语法 | 示例 | 说明 |
|------|------|------|
| **简单搜索** | `system monitor` | 搜索包含关键词的插件 |
| **精确匹配** | `"system monitor"` | 精确匹配短语 |
| **布尔操作** | `system AND monitor` | 必须包含两个词 |
| **布尔操作** | `system OR monitoring` | 包含任一词 |
| **排除** | `system -windows` | 包含system但不包含windows |
| **通配符** | `sys*` | 以sys开头的词 |

### 高级语法

| 语法 | 示例 | 说明 |
|------|------|------|
| **字段搜索** | `author:geektools` | 在特定字段搜索 |
| **标签搜索** | `tag:system` | 搜索特定标签 |
| **范围搜索** | `downloads:[100 TO 1000]` | 数值范围搜索 |
| **日期搜索** | `created:[2024-01-01 TO *]` | 日期范围搜索 |
| **存在性** | `homepage:*` | 字段存在性检查 |

### 可搜索字段

| 字段名 | 类型 | 示例 | 权重 |
|--------|------|------|------|
| `name` | 文本 | `name:monitor` | 高 |
| `description` | 文本 | `description:system` | 中 |
| `author` | 文本 | `author:geektools` | 中 |
| `tags` | 数组 | `tags:utility` | 中 |
| `license` | 关键词 | `license:MIT` | 低 |
| `status` | 关键词 | `status:active` | 低 |
| `downloads` | 数值 | `downloads:[100 TO *]` | 低 |
| `rating` | 数值 | `rating:[4.0 TO 5.0]` | 低 |

## 搜索优化

### 相关度计算

搜索结果的相关度基于以下因素计算：

1. **文本匹配度** (40%):
   - 关键词在标题中的匹配
   - 关键词在描述中的匹配
   - 词频和逆文档频率(TF-IDF)

2. **质量分数** (30%):
   - 插件评分
   - 下载量
   - 最近更新时间

3. **用户行为** (20%):
   - 点击率
   - 下载转化率
   - 用户反馈

4. **标签匹配** (10%):
   - 标签完全匹配
   - 标签相关性

### 搜索性能优化

- **索引优化**: 使用全文索引加速搜索
- **缓存策略**: 热门查询结果缓存
- **分页限制**: 限制单次查询结果数量
- **超时控制**: 复杂查询自动超时

## 示例代码

### JavaScript搜索客户端

```javascript
class SearchClient {
  constructor(baseUrl, authToken = null) {
    this.baseUrl = baseUrl;
    this.authToken = authToken;
  }

  // 高级搜索
  async advancedSearch(searchParams) {
    const response = await fetch(`${this.baseUrl}/search/advanced`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(this.authToken && { 'Authorization': `Bearer ${this.authToken}` }),
      },
      body: JSON.stringify(searchParams),
    });
    return await response.json();
  }

  // 搜索建议
  async getSuggestions(query, limit = 10) {
    const params = new URLSearchParams({ q: query, limit });
    const response = await fetch(`${this.baseUrl}/search/suggestions?${params}`);
    return await response.json();
  }

  // 热门搜索
  async getTrending(period = 'day', limit = 10) {
    const params = new URLSearchParams({ period, limit });
    const response = await fetch(`${this.baseUrl}/search/trending?${params}`);
    return await response.json();
  }

  // 搜索历史
  async getSearchHistory(limit = 20) {
    if (!this.authToken) {
      throw new Error('Authentication required for search history');
    }
    
    const params = new URLSearchParams({ limit });
    const response = await fetch(`${this.baseUrl}/search/history?${params}`, {
      headers: {
        'Authorization': `Bearer ${this.authToken}`,
      },
    });
    return await response.json();
  }

  // 实时搜索组件
  createSearchBox(container, options = {}) {
    const searchBox = document.createElement('div');
    searchBox.className = 'search-box';
    
    const input = document.createElement('input');
    input.type = 'text';
    input.placeholder = options.placeholder || 'Search plugins...';
    input.className = 'search-input';
    
    const suggestions = document.createElement('div');
    suggestions.className = 'search-suggestions';
    suggestions.style.display = 'none';
    
    let debounceTimeout;
    
    input.addEventListener('input', (e) => {
      const query = e.target.value;
      
      clearTimeout(debounceTimeout);
      debounceTimeout = setTimeout(async () => {
        if (query.length >= 2) {
          try {
            const result = await this.getSuggestions(query);
            this.displaySuggestions(suggestions, result.data.suggestions);
            suggestions.style.display = 'block';
          } catch (error) {
            console.error('Failed to fetch suggestions:', error);
          }
        } else {
          suggestions.style.display = 'none';
        }
      }, 300);
    });
    
    searchBox.appendChild(input);
    searchBox.appendChild(suggestions);
    container.appendChild(searchBox);
    
    return {
      input,
      search: (query) => this.advancedSearch({ query })
    };
  }

  displaySuggestions(container, suggestions) {
    container.innerHTML = '';
    
    suggestions.forEach(suggestion => {
      const item = document.createElement('div');
      item.className = 'suggestion-item';
      item.innerHTML = `
        <span class="suggestion-text">${suggestion.highlight}</span>
        <span class="suggestion-type">${suggestion.type}</span>
        <span class="suggestion-count">${suggestion.match_count}</span>
      `;
      
      item.addEventListener('click', () => {
        // 处理建议点击
        console.log('Selected suggestion:', suggestion.text);
      });
      
      container.appendChild(item);
    });
  }
}

// 使用示例
const searchClient = new SearchClient('http://localhost:3000/api/v1', 'user-token');

// 创建搜索界面
const searchContainer = document.getElementById('search-container');
const searchBox = searchClient.createSearchBox(searchContainer, {
  placeholder: 'Search GeekTools plugins...'
});

// 执行高级搜索
async function performSearch() {
  const searchParams = {
    query: 'system monitor',
    filters: {
      tags: ['system'],
      rating: { min: 4.0 },
      status: 'active'
    },
    sort: {
      field: 'downloads',
      order: 'desc'
    },
    pagination: {
      page: 1,
      limit: 20
    },
    highlight: true,
    include_stats: true
  };

  try {
    const results = await searchClient.advancedSearch(searchParams);
    console.log('Search results:', results);
    
    // 显示搜索结果
    displaySearchResults(results.data.plugins);
    
    // 显示搜索统计
    console.log('Search stats:', results.data.search_stats);
    
  } catch (error) {
    console.error('Search failed:', error);
  }
}

function displaySearchResults(plugins) {
  const resultsContainer = document.getElementById('results-container');
  resultsContainer.innerHTML = '';
  
  plugins.forEach(plugin => {
    const pluginCard = document.createElement('div');
    pluginCard.className = 'plugin-card';
    pluginCard.innerHTML = `
      <h3>${plugin.name}</h3>
      <p>${plugin.description}</p>
      <div class="plugin-meta">
        <span>Rating: ${plugin.rating}</span>
        <span>Downloads: ${plugin.downloads}</span>
        <span>Author: ${plugin.author}</span>
      </div>
      <div class="plugin-tags">
        ${plugin.tags.map(tag => `<span class="tag">${tag}</span>`).join('')}
      </div>
    `;
    
    resultsContainer.appendChild(pluginCard);
  });
}
```

### Python搜索脚本

```python
import requests
import json
from typing import Dict, List, Optional

class SearchClient:
    def __init__(self, base_url: str, auth_token: Optional[str] = None):
        self.base_url = base_url
        self.auth_token = auth_token
        self.session = requests.Session()
        
        if auth_token:
            self.session.headers.update({
                'Authorization': f'Bearer {auth_token}'
            })

    def advanced_search(self, **params) -> Dict:
        """执行高级搜索"""
        response = self.session.post(
            f'{self.base_url}/search/advanced',
            json=params
        )
        return response.json()

    def get_suggestions(self, query: str, limit: int = 10) -> Dict:
        """获取搜索建议"""
        params = {'q': query, 'limit': limit}
        response = self.session.get(
            f'{self.base_url}/search/suggestions',
            params=params
        )
        return response.json()

    def get_trending(self, period: str = 'day', limit: int = 10) -> Dict:
        """获取热门搜索"""
        params = {'period': period, 'limit': limit}
        response = self.session.get(
            f'{self.base_url}/search/trending',
            params=params
        )
        return response.json()

    def search_plugins(self, query: str, **filters) -> List[Dict]:
        """简化的插件搜索"""
        search_params = {
            'query': query,
            'filters': filters,
            'sort': {'field': 'relevance', 'order': 'desc'},
            'pagination': {'page': 1, 'limit': 50}
        }
        
        result = self.advanced_search(**search_params)
        if result['success']:
            return result['data']['plugins']
        else:
            raise Exception(f"Search failed: {result['error']}")

# 使用示例
search_client = SearchClient('http://localhost:3000/api/v1', 'user-token')

# 搜索系统监控插件
plugins = search_client.search_plugins(
    query='system monitor',
    tags=['system', 'monitoring'],
    rating={'min': 4.0},
    status='active'
)

print(f"找到 {len(plugins)} 个插件:")
for plugin in plugins:
    print(f"- {plugin['name']} (评分: {plugin['rating']}, 下载: {plugin['downloads']})")

# 获取搜索建议
suggestions = search_client.get_suggestions('sys')
print("\n搜索建议:")
for suggestion in suggestions['data']['suggestions']:
    print(f"- {suggestion['text']} ({suggestion['type']})")

# 获取热门搜索
trending = search_client.get_trending()
print("\n热门搜索:")
for trend in trending['data']['trending_searches']:
    print(f"- {trend['query']} ({trend['search_count']} 次搜索)")
```

## 常见问题

### Q: 搜索结果为什么不准确？
A: 可以尝试使用精确匹配语法 `"keyword"`，或者使用字段搜索如 `name:keyword` 来提高精确度。

### Q: 如何搜索特定作者的所有插件？
A: 使用字段搜索语法：`author:"作者名称"` 或在filters中指定author参数。

### Q: 搜索建议是基于什么生成的？
A: 基于插件名称、描述、标签、热门搜索词和用户搜索历史（如果已登录）。

### Q: 如何实现搜索结果的个性化？
A: 登录用户的搜索会基于历史行为、偏好标签等进行个性化排序。

### Q: 搜索是否支持中文？
A: 是的，支持中文全文搜索，包括中文分词和拼音搜索。