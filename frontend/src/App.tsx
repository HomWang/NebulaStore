import { useEffect, useMemo, useState } from 'react'

type BlockView = {
  height: number
  block_id: string
  timestamp: number
  tx_count: number
  data_gb: number
  status: string
}

type BlocksResponse = {
  protocol: string
  total_height: number
  blocks: BlockView[]
}

type ObjectView = {
  data_id: string
  content_hash: string
  content_type: string
  preview_url?: string | null
  file_name?: string | null
  file_size?: string | null
  file_type?: string | null
  file_preview?: string | null
  owner: string
  deployment: string
  stored_data: string
  created_at: number
  expires_at: number
  target_p95_ms: number
  min_replicas: number
  node_clusters: string[]
}

type ChampionRanking = {
  template: string
  composite_score: number
  predicted_p95_ms: number
  predicted_monthly_cost_usd: number
  reliability_score: number
  risk_level: string
}

type ChampionResponse = {
  champion_template: string
  strategy: string
  ranking: ChampionRanking[]
}

type UploadResponse = {
  message: string
  object: ObjectView
}

type ChunkInitResponse = {
  upload_id: string
  total_chunks: number
}

function formatTs(ts: number): string {
  if (!ts) return '-'
  return new Date(ts * 1000).toLocaleString('zh-CN')
}

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await fetch(url, init)
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${url}`)
  }
  return response.json() as Promise<T>
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = ''
  for (let i = 0; i < bytes.length; i += 1) {
    binary += String.fromCharCode(bytes[i])
  }
  return btoa(binary)
}

function isImageType(contentType: string): boolean {
  return contentType.toLowerCase().startsWith('image/')
}

function isVideoType(contentType: string): boolean {
  return contentType.toLowerCase().startsWith('video/')
}

function parseSizeToBytes(sizeLabel?: string | null): number {
  if (!sizeLabel) return 0
  const raw = sizeLabel.trim().toUpperCase()
  if (!raw) return 0

  const match = raw.match(/^([0-9]+(?:\.[0-9]+)?)\s*(B|KB|MB|GB|TB)$/)
  if (!match) return 0

  const value = Number(match[1])
  if (Number.isNaN(value) || value < 0) return 0

  const unit = match[2]
  const unitPow: Record<string, number> = {
    B: 0,
    KB: 1,
    MB: 2,
    GB: 3,
    TB: 4,
  }

  const pow = unitPow[unit] ?? 0
  return value * 1024 ** pow
}

export default function App() {
  const [page, setPage] = useState<'dashboard' | 'data'>('dashboard')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [blocks, setBlocks] = useState<BlockView[]>([])
  const [objects, setObjects] = useState<ObjectView[]>([])
  const [champion, setChampion] = useState<ChampionResponse | null>(null)
  const [query, setQuery] = useState('')
  const [queryResult, setQueryResult] = useState<ObjectView | null>(null)
  const [selectedData, setSelectedData] = useState<ObjectView | null>(null)
  const [selectedBlock, setSelectedBlock] = useState<BlockView | null>(null)
  const [mediaPreviewData, setMediaPreviewData] = useState<ObjectView | null>(null)
  const [uploadModalOpen, setUploadModalOpen] = useState(false)
  const [totalHeight, setTotalHeight] = useState(0)
  const [uploadOwner, setUploadOwner] = useState('alice')
  const [uploadName, setUploadName] = useState('dataset-demo')
  const [uploadType, setUploadType] = useState('application/json')
  const [uploadMode, setUploadMode] = useState<'editor' | 'file'>('editor')
  const [uploadContent, setUploadContent] = useState('{\n  "title": "demo",\n  "items": [1, 2, 3]\n}')
  const [selectedFile, setSelectedFile] = useState<File | null>(null)
  const [chunkSizeMB, setChunkSizeMB] = useState(2)
  const [dragOver, setDragOver] = useState(false)
  const [uploadBusy, setUploadBusy] = useState(false)
  const [uploadMessage, setUploadMessage] = useState('')
  const [copyMessage, setCopyMessage] = useState('')

  const totalData = useMemo(() => {
    const totalBytes = objects.reduce((sum, obj) => sum + parseSizeToBytes(obj.file_size), 0)
    return totalBytes / (1024 ** 3)
  }, [objects])

  const filteredObjects = useMemo(() => {
    const key = query.trim().toLowerCase()
    if (!key) return objects
    return objects.filter(
      (obj) =>
        obj.data_id.toLowerCase().includes(key) ||
        obj.content_hash.toLowerCase().includes(key) ||
        obj.stored_data.toLowerCase().includes(key),
    )
  }, [objects, query])

  const runDataQuery = async () => {
    const key = query.trim()
    if (!key) {
      setQueryResult(null)
      return
    }

    // 对 data_id/content_hash 走后端精确查询，其他关键词走前端本地筛选。
    if (key.startsWith('nebula-')) {
      try {
        const result = await fetchJson<ObjectView>(`/api/data/objects/${key}`)
        setQueryResult(result)
        return
      } catch {
        setQueryResult(null)
      }
    }

    if (/^[a-f0-9]{64}$/i.test(key)) {
      try {
        const result = await fetchJson<ObjectView>(`/api/data/objects/by-hash/${key.toLowerCase()}`)
        setQueryResult(result)
        return
      } catch {
        setQueryResult(null)
      }
    }

    const first = filteredObjects[0] ?? null
    setQueryResult(first)
  }

  const onFilePick = (file: File | null) => {
    setSelectedFile(file)
    if (file) {
      setUploadType(file.type || 'application/octet-stream')
      if (!uploadName.trim()) {
        setUploadName(file.name)
      }
    }
  }

  const runUpload = async () => {
    const owner = uploadOwner.trim()
    const dataName = uploadName.trim()
    const content = uploadContent
    if (!owner || !dataName || !content.trim()) {
      setUploadMessage('owner/data_name/content 不能为空')
      return
    }

    try {
      const res = await fetchJson<UploadResponse>('/api/data/upload', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          owner,
          data_name: dataName,
          content,
          content_type: uploadType.trim() || 'application/json',
        }),
      })
      setUploadMessage(`上传成功: ${res.object.data_id}`)
      setQuery(res.object.data_id)
      setQueryResult(res.object)
      await loadDashboard()
      setUploadModalOpen(false)
    } catch (err) {
      setUploadMessage(err instanceof Error ? `上传失败: ${err.message}` : '上传失败')
    }
  }

  const runFileUpload = async () => {
    const owner = uploadOwner.trim()
    const dataName = uploadName.trim()
    if (!owner || !dataName || !selectedFile) {
      setUploadMessage('owner/data_name/文件 不能为空')
      return
    }

    try {
      setUploadBusy(true)
      const form = new FormData()
      form.append('owner', owner)
      form.append('data_name', dataName)
      form.append('content_type', uploadType.trim())
      form.append('file', selectedFile)

      const response = await fetch('/api/data/upload/file', {
        method: 'POST',
        body: form,
      })

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: /api/data/upload/file`)
      }

      const res = (await response.json()) as UploadResponse
      setUploadMessage(`文件上传成功: ${res.object.data_id}`)
      setQuery(res.object.data_id)
      setQueryResult(res.object)
      await loadDashboard()
      setUploadModalOpen(false)
    } catch (err) {
      setUploadMessage(err instanceof Error ? `文件上传失败: ${err.message}` : '文件上传失败')
    } finally {
      setUploadBusy(false)
    }
  }

  const runChunkUpload = async () => {
    const owner = uploadOwner.trim()
    const dataName = uploadName.trim()
    const file = selectedFile
    if (!owner || !dataName || !file) {
      setUploadMessage('owner/data_name/文件 不能为空')
      return
    }

    const chunkBytes = Math.max(256 * 1024, Math.floor(chunkSizeMB * 1024 * 1024))
    const totalChunks = Math.ceil(file.size / chunkBytes)

    try {
      setUploadBusy(true)
      setUploadMessage(`分片初始化中... 共 ${totalChunks} 片`)

      const init = await fetchJson<ChunkInitResponse>('/api/data/upload/chunk/init', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          owner,
          data_name: dataName,
          file_name: file.name,
          total_chunks: totalChunks,
          content_type: uploadType.trim() || null,
        }),
      })

      for (let i = 0; i < totalChunks; i += 1) {
        const start = i * chunkBytes
        const end = Math.min(file.size, start + chunkBytes)
        const part = file.slice(start, end)
        const buf = new Uint8Array(await part.arrayBuffer())
        const chunkBase64 = bytesToBase64(buf)

        await fetchJson('/api/data/upload/chunk/part', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            upload_id: init.upload_id,
            chunk_index: i,
            chunk_base64: chunkBase64,
          }),
        })

        setUploadMessage(`分片上传中: ${i + 1}/${totalChunks}`)
      }

      const done = await fetchJson<UploadResponse>('/api/data/upload/chunk/complete', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ upload_id: init.upload_id }),
      })

      setUploadMessage(`分片上传完成: ${done.object.data_id}`)
      setQuery(done.object.data_id)
      setQueryResult(done.object)
      await loadDashboard()
      setUploadModalOpen(false)
    } catch (err) {
      setUploadMessage(err instanceof Error ? `分片上传失败: ${err.message}` : '分片上传失败')
    } finally {
      setUploadBusy(false)
    }
  }

  const loadDashboard = async () => {
    try {
      setLoading(true)
      setError(null)

      const [blockData, objectData, championData] = await Promise.all([
        fetchJson<BlocksResponse>('/api/chain/blocks'),
        fetchJson<{ protocol: string; objects: ObjectView[] }>('/api/data/objects'),
        fetchJson<ChampionResponse>('/api/protocol/template/champion', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ size_gb: 120, max_monthly_budget_usd: 3000, strategy: 'balanced' }),
        }),
      ])

      setBlocks(blockData.blocks)
      setTotalHeight(blockData.total_height)
      setObjects(objectData.objects)
      setChampion(championData)
    } catch (err) {
      setError(err instanceof Error ? err.message : '加载失败')
    } finally {
      setLoading(false)
    }
  }

  const copyText = async (label: string, text: string) => {
    try {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        await navigator.clipboard.writeText(text)
      } else {
        const textarea = document.createElement('textarea')
        textarea.value = text
        textarea.style.position = 'fixed'
        textarea.style.opacity = '0'
        document.body.appendChild(textarea)
        textarea.focus()
        textarea.select()
        document.execCommand('copy')
        document.body.removeChild(textarea)
      }
      setCopyMessage(`已复制${label}`)
      window.setTimeout(() => setCopyMessage(''), 1400)
    } catch {
      setCopyMessage(`复制失败：${label}`)
      window.setTimeout(() => setCopyMessage(''), 1800)
    }
  }

  const openMediaPreview = (obj: ObjectView) => {
    if (!obj.preview_url) return
    setSelectedData(obj)
    setMediaPreviewData(obj)
  }

  const renderFilePreview = (obj: ObjectView) => {
    const label = obj.file_preview || '-'
    const marker = '详情中查看'
    const markerIndex = label.indexOf(marker)

    if (!obj.preview_url || markerIndex < 0) {
      return label
    }

    const prefix = label.slice(0, markerIndex)
    const suffix = label.slice(markerIndex + marker.length)

    return (
      <span>
        {prefix}
        <button className="preview-inline-btn" onClick={() => openMediaPreview(obj)}>
          预览
        </button>
        {suffix}
      </span>
    )
  }

  useEffect(() => {
    void loadDashboard()
  }, [])

  return (
    <div className="page">
      <header className="hero">
        <div>
          <p className="eyebrow">NebulaStore</p>
          <h1>去中心化存储运营总览</h1>
          <p className="sub">总览页看网络状态，数据信息页看“具体存了什么数据”</p>
        </div>
        <div className="actions">
          <button
            className={page === 'dashboard' ? 'tab active' : 'tab'}
            onClick={() => setPage('dashboard')}
          >
            总览页
          </button>
          <button className={page === 'data' ? 'tab active' : 'tab'} onClick={() => setPage('data')}>
            数据信息页
          </button>
          <button className="refresh" onClick={() => void loadDashboard()} disabled={loading}>
            {loading ? '刷新中...' : '刷新数据'}
          </button>
        </div>
      </header>

      {error ? <div className="error">请求失败: {error}</div> : null}

      {page === 'dashboard' ? (
        <>
          <section className="kpi-grid">
            <article className="kpi">
              <span>区块数量</span>
              <strong>{blocks.length}</strong>
            </article>
            <article className="kpi">
              <span>对象数量</span>
              <strong>{objects.length}</strong>
            </article>
            <article className="kpi">
              <span>估算数据规模</span>
              <strong>{totalData.toFixed(2)} GB</strong>
            </article>
            <article className="kpi">
              <span>区块总高度</span>
              <strong>{totalHeight}</strong>
            </article>
          </section>

          <section className="panel">
            <div className="panel-head">
              <h2>策略冠军</h2>
              <span>{champion?.strategy ?? '-'}</span>
            </div>
            <p className="champion-name">{champion?.champion_template ?? '暂无'}</p>
            <div className="chips">
              {champion?.ranking?.slice(0, 3).map((item) => (
                <span key={item.template} className="chip">
                  {item.template}: {item.composite_score}
                </span>
              ))}
            </div>
          </section>

          <section className="panel">
            <div className="panel-head">
              <h2>区块信息</h2>
              <span>总高度 {totalHeight} · 最近 {blocks.length} 条</span>
            </div>
            <p className="metric-note">
              `finalized` 表示区块在当前演示系统中已确认不可回滚；`数据量(GB)`为演示估算值，不是链上真实观测值。
            </p>
            <div className="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>高度</th>
                    <th>区块 ID</th>
                    <th>时间</th>
                    <th>交易数</th>
                    <th>数据量(GB)</th>
                    <th>状态</th>
                    <th>详情</th>
                  </tr>
                </thead>
                <tbody>
                  {blocks.map((b) => (
                    <tr key={b.block_id}>
                      <td>{b.height}</td>
                      <td>{b.block_id}</td>
                      <td>{formatTs(b.timestamp)}</td>
                      <td>{b.tx_count}</td>
                      <td>{b.data_gb.toFixed(2)}</td>
                      <td>{b.status}</td>
                      <td>
                        <button
                          className="small-btn"
                          onClick={() => {
                            setSelectedBlock(b)
                          }}
                        >
                          查看详情
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </section>
        </>
      ) : (
        <section className="panel">
          <div className="panel-head">
            <h2>数据信息</h2>
            <span>对象视图（支持查询与详情）</span>
          </div>

          <div className="upload-entry">
            <button
              onClick={() => {
                setUploadMessage('')
                setUploadModalOpen(true)
              }}
            >
              数据上传
            </button>
            <span>支持数据编辑上传、文件上传、分片上传</span>
          </div>

          <div className="query-bar">
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="输入数据ID / 64位内容哈希 / 关键词（部署名、存储内容）"
            />
            <button onClick={() => void runDataQuery()}>查询</button>
          </div>

          {query.trim() ? (
            <p className="query-hint">当前匹配：{filteredObjects.length} 条</p>
          ) : null}

          {queryResult ? (
            <div className="result-card">
              <h3>
                查询结果：
                <span className="copy-line">
                  {queryResult.data_id}
                  <button
                    className="icon-btn"
                    title="复制数据ID"
                    aria-label="复制数据ID"
                    onClick={() => void copyText('数据ID', queryResult.data_id)}
                  >
                    <span className="copy-icon" />
                  </button>
                </span>
              </h3>
              <p>
                <strong>内容哈希：</strong>
                <span className="copy-line">
                  {queryResult.content_hash}
                  <button
                    className="icon-btn"
                    title="复制内容哈希"
                    aria-label="复制内容哈希"
                    onClick={() => void copyText('内容哈希', queryResult.content_hash)}
                  >
                    <span className="copy-icon" />
                  </button>
                </span>
              </p>
              <p>
                <strong>存储数据内容：</strong>
                {queryResult.stored_data}
              </p>
              <p>
                <strong>文件名：</strong>
                {queryResult.file_name || '-'}
              </p>
              <p>
                <strong>大小：</strong>
                {queryResult.file_size || '-'}
              </p>
              <p>
                <strong>类型：</strong>
                {queryResult.file_type || '-'}
              </p>
              <p>
                <strong>预览：</strong>
                {renderFilePreview(queryResult)}
              </p>
              <p>
                <strong>节点池：</strong>
                {queryResult.node_clusters.join(', ')}
              </p>
              <button
                className="small-btn"
                onClick={() => {
                  setSelectedData(queryResult)
                  setMediaPreviewData(null)
                }}
              >
                查看数据详情
              </button>
            </div>
          ) : null}

          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>数据ID</th>
                  <th>内容哈希</th>
                  <th>文件名</th>
                  <th>大小</th>
                  <th>类型</th>
                  <th>预览</th>
                  <th>节点池</th>
                  <th>详情</th>
                </tr>
              </thead>
              <tbody>
                {filteredObjects.map((obj) => (
                  <tr key={obj.data_id}>
                    <td>
                      <span className="copy-line">
                        {obj.data_id}
                        <button
                          className="icon-btn"
                          title="复制数据ID"
                          aria-label="复制数据ID"
                          onClick={() => void copyText('数据ID', obj.data_id)}
                        >
                          <span className="copy-icon" />
                        </button>
                      </span>
                    </td>
                    <td>
                      <span className="copy-line">
                        {obj.content_hash.slice(0, 12)}...
                        <button
                          className="icon-btn"
                          title="复制内容哈希"
                          aria-label="复制内容哈希"
                          onClick={() => void copyText('内容哈希', obj.content_hash)}
                        >
                          <span className="copy-icon" />
                        </button>
                      </span>
                    </td>
                    <td>{obj.file_name || '-'}</td>
                    <td>{obj.file_size || '-'}</td>
                    <td>{obj.file_type || '-'}</td>
                    <td>{renderFilePreview(obj)}</td>
                    <td>{obj.node_clusters.join(', ')}</td>
                    <td>
                      <button
                        className="small-btn"
                        onClick={() => {
                          setSelectedData(obj)
                          setMediaPreviewData(null)
                        }}
                      >
                        查看详情
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      )}

      {uploadModalOpen ? (
        <div className="modal-overlay" onClick={() => setUploadModalOpen(false)}>
          <div className="modal-card" onClick={(e) => e.stopPropagation()}>
            <div className="modal-head">
              <h3>数据上传</h3>
              <button className="small-btn" onClick={() => setUploadModalOpen(false)}>
                关闭
              </button>
            </div>

            <div className="upload-card">
              <div className="upload-mode-tabs">
                <button
                  className={uploadMode === 'editor' ? 'upload-mode-tab active' : 'upload-mode-tab'}
                  onClick={() => setUploadMode('editor')}
                >
                  数据编辑上传
                </button>
                <button
                  className={uploadMode === 'file' ? 'upload-mode-tab active' : 'upload-mode-tab'}
                  onClick={() => setUploadMode('file')}
                >
                  文件/分片上传
                </button>
              </div>
              <div className="upload-grid">
                <input value={uploadOwner} onChange={(e) => setUploadOwner(e.target.value)} placeholder="owner" />
                <input value={uploadName} onChange={(e) => setUploadName(e.target.value)} placeholder="data_name" />
                <input
                  value={uploadType}
                  onChange={(e) => setUploadType(e.target.value)}
                  placeholder="content_type (application/json)"
                />
              </div>
              {uploadMode === 'editor' ? (
                <>
                  <textarea
                    className="json-editor"
                    value={uploadContent}
                    onChange={(e) => setUploadContent(e.target.value)}
                    placeholder="在这里粘贴或编辑 JSON / 文本内容，后端将按 content 生成内容哈希"
                  />
                  <div className="upload-actions">
                    <button onClick={() => void runUpload()}>JSON 上传</button>
                    {uploadMessage ? <span>{uploadMessage}</span> : null}
                  </div>
                </>
              ) : (
                <>
                  <div
                    className={dragOver ? 'dropzone active' : 'dropzone'}
                    onDragOver={(e) => {
                      e.preventDefault()
                      setDragOver(true)
                    }}
                    onDragLeave={() => setDragOver(false)}
                    onDrop={(e) => {
                      e.preventDefault()
                      setDragOver(false)
                      const file = e.dataTransfer.files?.[0] ?? null
                      onFilePick(file)
                    }}
                  >
                    <p>{selectedFile ? `已选择: ${selectedFile.name}` : '拖拽文件到这里上传'}</p>
                    <label className="file-picker">
                      选择文件
                      <input
                        type="file"
                        onChange={(e) => onFilePick(e.target.files?.[0] ?? null)}
                        style={{ display: 'none' }}
                      />
                    </label>
                  </div>

                  <div className="chunk-row">
                    <span>分片大小(MB)</span>
                    <input
                      type="number"
                      min={1}
                      max={20}
                      value={chunkSizeMB}
                      onChange={(e) => setChunkSizeMB(Number(e.target.value) || 2)}
                    />
                  </div>

                  <div className="upload-actions">
                    <button onClick={() => void runFileUpload()} disabled={uploadBusy}>
                      {uploadBusy ? '处理中...' : '文件上传'}
                    </button>
                    <button onClick={() => void runChunkUpload()} disabled={uploadBusy}>
                      {uploadBusy ? '处理中...' : '分片上传'}
                    </button>
                    {uploadMessage ? <span>{uploadMessage}</span> : null}
                  </div>
                </>
              )}
            </div>
          </div>
        </div>
      ) : null}

      {selectedData ? (
        <div
          className="modal-overlay"
          onClick={() => {
            setSelectedData(null)
            setMediaPreviewData(null)
          }}
        >
          <div className="modal-card" onClick={(e) => e.stopPropagation()}>
            <div className="detail-card">
              <div className="detail-head">
                <h3>数据详情：{selectedData.data_id}</h3>
                <button
                  className="small-btn"
                  onClick={() => {
                    setSelectedData(null)
                    setMediaPreviewData(null)
                  }}
                >
                  关闭
                </button>
              </div>
              <p>
                <strong>数据ID：</strong>
                <span className="copy-line">
                  {selectedData.data_id}
                  <button
                    className="icon-btn"
                    title="复制数据ID"
                    aria-label="复制数据ID"
                    onClick={() => void copyText('数据ID', selectedData.data_id)}
                  >
                    <span className="copy-icon" />
                  </button>
                </span>
              </p>
              <p>
                <strong>内容哈希：</strong>
                <span className="copy-line">
                  {selectedData.content_hash}
                  <button
                    className="icon-btn"
                    title="复制内容哈希"
                    aria-label="复制内容哈希"
                    onClick={() => void copyText('内容哈希', selectedData.content_hash)}
                  >
                    <span className="copy-icon" />
                  </button>
                </span>
              </p>
              <p>
                <strong>内容类型：</strong>
                {selectedData.content_type || 'application/json'}
              </p>
              <p>
                <strong>文件名：</strong>
                {selectedData.file_name || '-'}
              </p>
              <p>
                <strong>大小：</strong>
                {selectedData.file_size || '-'}
              </p>
              <p>
                <strong>类型：</strong>
                {selectedData.file_type || '-'}
              </p>
              <p>
                <strong>预览：</strong>
                {renderFilePreview(selectedData)}
              </p>
              <div>
                <strong>存储数据内容：</strong>
                <div className="detail-content-box">{selectedData.stored_data}</div>
              </div>
              <p>
                <strong>创建时间：</strong>
                {formatTs(selectedData.created_at)}
              </p>
              <p>
                <strong>到期时间：</strong>
                {formatTs(selectedData.expires_at)}
              </p>
              <p>
                <strong>节点池：</strong>
                {selectedData.node_clusters.join(', ')}
              </p>

            </div>
          </div>
        </div>
      ) : null}

      {mediaPreviewData?.preview_url ? (
        <div className="modal-overlay preview-overlay" onClick={() => setMediaPreviewData(null)}>
          <div className="modal-card preview-modal-card" onClick={(e) => e.stopPropagation()}>
            <div className="detail-head">
              <h3>媒体预览：{mediaPreviewData.data_id}</h3>
              <button className="small-btn" onClick={() => setMediaPreviewData(null)}>
                关闭预览
              </button>
            </div>

            {isImageType(mediaPreviewData.content_type || '') ? (
              <img src={mediaPreviewData.preview_url} alt={mediaPreviewData.data_id} className="media-preview" />
            ) : null}

            {isVideoType(mediaPreviewData.content_type || '') ? (
              <video src={mediaPreviewData.preview_url} controls className="media-preview" />
            ) : null}
          </div>
        </div>
      ) : null}

      {selectedBlock ? (
        <div className="modal-overlay" onClick={() => setSelectedBlock(null)}>
          <div className="modal-card" onClick={(e) => e.stopPropagation()}>
            <div className="detail-card">
              <div className="detail-head">
                <h3>区块详情：{selectedBlock.block_id}</h3>
                <button className="small-btn" onClick={() => setSelectedBlock(null)}>
                  关闭
                </button>
              </div>
              <p>
                <strong>区块高度：</strong>
                {selectedBlock.height}
              </p>
              <p>
                <strong>链总高度：</strong>
                {totalHeight}
              </p>
              <p>
                <strong>出块时间：</strong>
                {formatTs(selectedBlock.timestamp)}
              </p>
              <p>
                <strong>交易总数：</strong>
                {selectedBlock.tx_count}
              </p>
              <p>
                <strong>本块数据量：</strong>
                {selectedBlock.data_gb.toFixed(2)} GB（估算）
              </p>
              <p>
                <strong>确认状态：</strong>
                {selectedBlock.status}（已确认）
              </p>
            </div>
          </div>
        </div>
      ) : null}

      {copyMessage ? <div className="copy-toast">{copyMessage}</div> : null}
    </div>
  )
}
