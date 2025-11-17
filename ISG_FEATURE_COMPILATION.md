# Infinite Storage Glitch - Comprehensive Feature Compilation

*Compiled from 28 different ISG implementations found on GitHub*

## CORE ENCODING/DECODING TECHNIQUES

### Binary Representation Methods:
- **Black/white pixel encoding** (0=white, 1=black) - KKarmugil, Atiseug, Rohit10701, multiple others
- **Black/white with blue EOF marker** - Memorix101
- **2x2 pixel blocks** for compression resistance - KKarmugil, OM-bit-hub
- **3x3 pixel blocks** for redundancy - Rohit10701, dev2180, alexanki23890t
- **4x4 pixel blocks** (16 pixels per bit) - Memorix101, KKarmugil, Thats-Not-A-Vid
- **6x6 pixel blocks with frame numbering** - Rohit10701
- **10x10 pixel blocks** - User1334
- **Color-based Base64 encoding** (assign unique RGB values to Base64 characters) - knkr1

### Encoding Algorithms:
- **Direct binary-to-pixel mapping** - Multiple implementations
- **Frame sequencing with embedded frame numbers** - Rohit10701
- **Sorted dictionary reconstruction** for frame ordering - Rohit10701
- **Threshold-based pixel reading** (default 128, adjustable to 200) - Multiple
- **Averaging pixel blocks** during decode - Multiple
- **Grayscale conversion with threshold** - KKarmugil, Atiseug
- **Cosine similarity for color assignment** - knkr1
- **Custom RGB assignment algorithm for Base64** - knkr1

---

## COMPRESSION & OPTIMIZATION

### Compression Methods:
- **GZip compression** - Memorix101
- **Zstd compression** (level 3) - Memorix101
- **ZIP compression** before encoding - Atiseug, Rohit10701
- **Bzip2 compression** for folders - norangeflame
- **Tar.bz2 for folder compression** - norangeflame
- **Automatic folder tarball creation** - norangeflame

### Performance Optimizations:
- **GPU acceleration** with PyTorch/Metal API (Apple Silicon M1/M2) - User1334
- **GPU acceleration** with h264_videotoolbox - User1334
- **Chunk-based file reading** (1024 bytes) - KKarmugil
- **Lazy generator for file chunks** - Atiseug
- **Progress bars** with tqdm - Multiple implementations
- **Memory-efficient streaming** - User1334
- **FFmpeg pipe streaming** for frame processing - User1334, knkr1
- **Batch frame processing** - Multiple
- **Configurable resize factors** (0-4x) - Atiseug
- **Dynamic resolution calculation** - knkr1

---

## FILE HANDLING & METADATA

### Metadata Preservation:
- **Embedded filename in video** - Memorix101 (ISGv2 header), Thats-Not-A-Vid, User1334
- **File size (EOF marker)** - Memorix101, Nick4421
- **Creation and modification timestamps** - User1334
- **JSON metadata embedding** - User1334
- **Magic number** (ISGv2) for version detection - Memorix101
- **File extension preservation** - Memorix101
- **Custom header at specific byte offsets** (64, 256, 784) - Memorix101

### File Splitting:
- **Automatic file splitting** for files >25MB (Discord limit) - norangeflame
- **Automatic file splitting** for files >24MB chunks - norangeflame
- **Automatic rejoining** of split files - norangeflame
- **Part numbering** (.part1, .part2, etc.) - norangeflame
- **Master record** tracking split files - norangeflame

---

## SUPPORTED PLATFORMS

### Upload/Storage Platforms:
- **YouTube** - Multiple implementations
- **Discord** via webhooks - norangeflame
- **Generic cloud storage** - Multiple
- **Local storage** - All implementations

### Platform-Specific Features:

#### YouTube Integration:
- **YouTube Data API v3** integration - Rohit10701, dev2180, alexanki23890t
- **YouTube OAuth2** authentication - Rohit10701, dev2180
- **Google OAuth flow** with InstalledAppFlow - Rohit10701
- **YouTube video download** with pytube - KKarmugil, OM-bit-hub
- **YouTube video update/edit** - Rohit10701 (update.py)
- **YouTube video deletion** - Rohit10701 (delete.py)
- **YouTube channel/playlist listing** - Rohit10701 (list.py)
- **Privacy status control** (unlisted/public) - Rohit10701
- **Video metadata** (title, tags, category) - Rohit10701
- **Resumable upload** sessions - Rohit10701
- **Chunked upload** with Content-Range headers - Rohit10701

#### Discord Integration:
- **Discord webhook** integration - norangeflame
- **Discord special character stripping** (spaces, special chars) - norangeflame

---

## VIDEO FORMATS & CODECS

### Output Formats:
- **MP4** (H.264) - Multiple
- **MP4** (VP9 codec) - Memorix101
- **MP4** (mp4v codec) - Thats-Not-A-Vid
- **AVI** - Thats-Not-A-Vid
- **GIF** (animated) - Memorix101
- **JPEG** sequence - Memorix101
- **PNG** sequence - Multiple
- **Raw PBM** (Portable Bitmap) - Nick4421
- **MKV** (FFV1 lossless codec) - knkr1
- **MOV** (H.264_videotoolbox) - User1334
- **WebM** support - KKarmugil

### Video Settings:
- **Configurable resolution** (640x360, 1280x720, 1920x1080) - Multiple
- **Custom resolution support** - Multiple
- **360p, 720p, 1080p** presets - Multiple
- **Configurable FPS** (1fps, 24fps, 30fps, 60fps) - Multiple
- **CRF 0** (lossless) - Memorix101
- **CRF 30** - Rohit10701
- **UltraFast encoder preset** - Memorix101
- **YUV444 color space** - Memorix101
- **Configurable block sizes** - Multiple
- **Frame duplication** (2 identical frames) - Thats-Not-A-Vid, Nick4421

---

## ERROR CORRECTION & DATA INTEGRITY

### Error Correction:
- **Error correction bits** (8-bit parity) - User1334
- **Checksum validation** - User1334
- **Blue pixel EOF marker** - Memorix101 (v0.2.0)
- **Metadata frame with EOF** - Memorix101 (v0.2.2+)
- **Frame scale-up** (2x) for compression resistance - Nick4421
- **Frame scale-down** with -nomix flag (majority voting) - Nick4421
- **Pixel block averaging** for noise reduction - Multiple
- **Undefined pixel detection** - Memorix101
- **Experimental mode** for inaccurate pixels - knkr1
- **Nearest-neighbor color matching** - knkr1
- **Euclidean distance** for color correction - knkr1

### Validation:
- **Tolerance levels** (200 default) for pixel reading - Memorix101
- **Empty frame detection** and removal - Memorix101
- **File integrity checks** - Multiple
- **Compression ratio tracking** - Memorix101
- **Pixel count validation** (black vs white) - Memorix101

---

## USER INTERFACES

### CLI Interfaces:
- **Simple numbered menu** (1/2/3 options) - KKarmugil, OM-bit-hub
- **Character-based menu** (e/d/q) - Nick4421
- **Prompt-based workflow** - Multiple
- **File path input** with validation - Nick4421
- **Directory validation** - Nick4421
- **Extension validation** (.mp4) - Nick4421
- **Interactive prompts** - Multiple
- **Clear/refresh terminal** - Memorix101
- **ANSI color codes** for output - Nick4421
- **Progress indicators** with tqdm - Multiple
- **Status updates** - Multiple
- **Error messages** - Multiple
- **Success confirmation** - Multiple

### GUI Interfaces:
- **Tkinter GUI** - norangeflame
- **File dialog** for file selection - norangeflame
- **Folder dialog** - norangeflame
- **Listbox** for file browsing - norangeflame
- **Status labels** - norangeflame
- **Progress indicators** in GUI - norangeflame
- **Download speed display** - norangeflame
- **File size display** - norangeflame
- **Button interface** - norangeflame
- **Config GUI** with entry fields - norangeflame

### Web Interfaces:
- **React frontend** - Rohit10701, dev2180, alexanki23890t
- **Flask REST API** - Rohit10701, dev2180, alexanki23890t
- **File upload endpoint** - Rohit10701
- **Axios HTTP client** - Rohit10701

---

## PROGRAMMING LANGUAGES & FRAMEWORKS

### Languages:
- **Python** (majority) - Multiple implementations
- **C#/.NET 6** - Memorix101
- **C** - Nick4421, Thats-Not-A-Vid
- **C++** - Thats-Not-A-Vid (unused/old code)
- **Rust** (original reference) - Referenced by multiple
- **JavaScript/React** - Rohit10701, dev2180

### Key Libraries & Frameworks:

#### Python:
- **PIL/Pillow** - Image manipulation (majority)
- **OpenCV** (cv2) - Video processing (multiple)
- **MoviePy** - Video creation (KKarmugil, OM-bit-hub)
- **imageio** - Frame extraction (KKarmugil, Atiseug)
- **NumPy** - Array operations (multiple)
- **tqdm** - Progress bars (multiple)
- **pytube** - YouTube downloads (KKarmugil, OM-bit-hub)
- **google-auth-oauthlib** - OAuth (Rohit10701)
- **googleapiclient** - YouTube API (Rohit10701)
- **discord-webhook** - Discord integration (norangeflame)
- **Flask** - Web API (Rohit10701)
- **PyTorch** - GPU acceleration (User1334)
- **natsort** - Natural sorting (Atiseug)
- **scikit-learn** - Cosine similarity (knkr1)
- **base64** - Encoding (knkr1)
- **json** - Configuration (multiple)
- **subprocess** - FFmpeg control (multiple)

#### C#:
- **FFMediaToolkit** - Video processing
- **ZstdSharp** - Zstd compression
- **ImageSharp** - Image manipulation
- **ImageSharp.Drawing** - Drawing operations
- **System.IO.Compression** - GZip

---

## EXTERNAL DEPENDENCIES

### Required Tools:
- **FFmpeg** - Video encoding/decoding (nearly all)
- **FFprobe** - Video analysis (some)
- **Netpbm tools** (pamenlarge, pamscale) - Nick4421
- **Docker** - Containerization (ycs77)

---

## UNIQUE/INNOVATIVE FEATURES

1. **Docker containerization** for easy deployment - ycs77
2. **Base64 + RGB color mapping** encoding - knkr1
3. **GPU acceleration** with Metal API (Apple Silicon) - User1334
4. **Metadata preservation** (timestamps) - User1334
5. **Discord webhook** integration - norangeflame
6. **Automatic folder compression** (.tar.bz2) - norangeflame
7. **File splitting** >25MB with automatic rejoining - norangeflame
8. **Master record** system for file tracking - norangeflame
9. **React + Flask** full-stack implementation - Rohit10701
10. **Raw PBM format** for efficiency - Nick4421
11. **Frame scale-up/down** for compression resistance - Nick4421
12. **Versioned metadata** (ISGv2 magic number) - Memorix101
13. **Multiple output formats** (GIF/JPEG/MP4) - Memorix101
14. **Compression offset** tracking - Memorix101
15. **Error correction bits** with parity - User1334
16. **Cosine similarity** for color uniqueness - knkr1
17. **Experimental color correction** mode - knkr1
18. **YouTube video management** (update/delete/list) - Rohit10701
19. **Frame numbering** in first 160 bits - Rohit10701
20. **Download speed tracking** - norangeflame

---

## CONFIGURATION OPTIONS

- **Configurable resolution** (multiple presets)
- **Configurable FPS** (1-60)
- **Configurable pixel block size** (2x2 to 10x10)
- **Configurable compression level** (Zstd 1-22)
- **Configurable CRF** (0-51)
- **Configurable encoder preset** (ultrafast to veryslow)
- **Config files** (INI, JSON) - norangeflame, knkr1
- **Environment variables** - Some implementations
- **Command-line arguments** - Multiple

---

## FILE MANAGEMENT FEATURES

- **Batch processing** - Potential in multiple
- **File deletion** from local - Multiple
- **File deletion** from cloud - Rohit10701
- **File listing** - Rohit10701
- **File browsing** UI - norangeflame
- **File filtering** by type - Multiple
- **Natural sorting** - Atiseug
- **Automatic cleanup** of temp files - Multiple
- **Directory creation** - Multiple
- **Duplicate detection** - norangeflame
- **File existence checking** - Multiple

---

## ADDITIONAL FEATURES

- **Multi-language support** potential (German in User1334)
- **Logging** with detailed output - Multiple
- **Error handling** with try/catch - Multiple
- **Verbose mode** - Some
- **Quiet mode** - Some
- **Resume capability** - YouTube resumable uploads (Rohit10701)
- **Webhook notifications** - norangeflame

---

## PLATFORMS & COMPATIBILITY

- **Windows** - Memorix101, multiple
- **Linux** - Multiple
- **macOS** - User1334, Nick4421, multiple
- **Cross-platform** - Most Python implementations
- **Apple Silicon** (M1/M2) - User1334
- **Docker** - ycs77

---

## Summary Statistics

- **Total Implementations Analyzed**: 28
- **Programming Languages**: Python, C#, C, C++, Rust, JavaScript
- **Supported Platforms**: YouTube, Discord, Local Storage
- **Video Formats**: MP4, AVI, GIF, JPEG, PNG, PBM, MKV, MOV, WebM
- **Compression Methods**: GZip, Zstd, ZIP, Bzip2, Tar
- **UI Types**: CLI, GUI (Tkinter), Web (React/Flask)
- **Unique Features Identified**: 200+

---

*This compilation provides a complete foundation for designing the most comprehensive and feature-rich Infinite Storage Glitch implementation ever created.*
