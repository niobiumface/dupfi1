# DupFi - Duplicate File Finder

DupFi is a  duplicate finder with a graphical interface that helps you find and manage duplicate files.

## Features

✅ **User-Friendly Interface**  
- Integrated file explorer for directory selection  
- Progress display for large scans  
- Clear overview of duplicates  

✅ **Duplicate Detection**  
- Fast detection using SHA256 hashing  
- Size-based pre-filtering for optimal performance  
- Multithreading for fast scans  

✅ **Flexible Management Options**  
- Delete duplicates  
- Create hard links for storage optimization  
- Move files  
- Preview for text and image files  

✅ **Filtering Options**  
- Exclude specific file types  
- Customizable filter rules  

## Installation

1. Download the latest version of DupFi  
2. Extract the ZIP file  
3. Run `dupfi.exe`  

## Usage

1. Click "📁 Select Directory" to choose a directory  
2. (Optional) Add filters to exclude certain file types  
3. Click "🔍 Start Scan" to begin the search  
4. Manage found duplicates using the available options:  
   - 🗑️ Delete  
   - 🔗 Create Hard Link  
   - 📦 Move  

## Technical Details

- Written in Rust  
- Uses egui for the user interface  
- Multithreading with rayon  
- Secure file management with error handling  

## Build from Source
