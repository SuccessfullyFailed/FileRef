use std::{error::Error, ops::{Add, AddAssign}};

use crate::FileScanner;



// Could be chars, but will be used as str's mainly, so this stops the program from converting.
pub(crate) const SEPARATOR:&str = "/";
const INVALID_SEPARATOR:&str = "\\";



#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileRef {
	StaticStr(&'static str),
	Owned(String)
}
impl FileRef {

	/* CONSTRUCTOR METHODS */

	/// Create a new owned path.
	pub fn new(path:&str) -> FileRef {
		FileRef::Owned(path.replace(INVALID_SEPARATOR, SEPARATOR))
	}

	/// Create a new statically borrowed path.
	pub const fn new_const(path:&'static str) -> FileRef {
		FileRef::StaticStr(path)
	}



	/* PROPERTY GETTER METHODS */

	/// Get the raw path.
	pub fn path(&self) -> &str {
		match self {
			FileRef::StaticStr(path) => *path,
			FileRef::Owned(path) => path.as_str()
		}
	}

	/// Get the directory the file is in.
	pub fn parent_dir(&self) -> Result<FileRef, Box<dyn Error>> {
		let path:&str = self.path();
		let nodes:Vec<&str> = self.path_nodes();
		if nodes.len() <= 1 {
			Err(format!("Could not get dir of file \"{path}\", as it only contains the file name.").into())
		} else {
			let parent_dir_len:usize = nodes[..nodes.len() - 1].join(SEPARATOR).len();
			Ok(FileRef::new(&path[..parent_dir_len]))
		}
	}

	/// Get a list of nodes in the path.
	pub(crate) fn path_nodes(&self) -> Vec<&str> {
		self.path().split(SEPARATOR).collect()
	}

	/// Get the last node of the path.
	pub(crate) fn last_node(&self) -> &str {
		self.path().split(SEPARATOR).last().unwrap_or_default()
	}



	/* PROPERTY GETTER METHODS */

	/// Check if self is a dir.
	pub fn is_dir(&self) -> bool {
		self.extension().map(|extension| extension.is_empty()).unwrap_or(true)
	}

	/// Check if self is a file.
	pub fn is_file(&self) -> bool {
		!self.is_dir()
	}

	/// Get the name of the file/dir.
	pub fn name(&self) -> &str {
		self.last_node()
	}

	/// Get the name of the file without extension.
	pub fn file_name_no_extension(&self) -> &str {
		self.name().trim_end_matches(&self.extension().map(|extension| (".".to_owned() + extension)).unwrap_or_default())
	}

	/// Get the extension of the file.
	pub fn extension(&self) -> Option<&str> {
		let file_name:&str = self.name();
		if file_name.contains('.') {
			file_name.split('.').last()
		} else {
			None
		}
	}

	/// Check if the files exists.
	pub fn exists(&self) -> bool {
		std::path::Path::new(&self.path()).exists() && std::fs::metadata(&self.path()).is_ok()
	}
	
	/// Check if the file can be accessed.
	pub fn is_accessible(&self) -> bool {
		if self.is_dir() { true } else { std::fs::File::open(&self.path()).is_ok() }
	}



	/* FILE READING METHODS */

	/// Read the contents of the file as a string.
	pub fn read(&self) -> Result<String, Box<dyn Error>> {
		use std::{ fs::File, io::Read };
		
		if self.is_dir() {
			Err(format!("Could not read dir \"{}\". Only able to read files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not read file \"{}\". File does not exist.", self.path()).into())
		} else {
			let mut file:File = File::open(self.path())?;
			let mut contents:String = String::new();
			file.read_to_string(&mut contents)?;
			Ok(contents)
		}
	}

	/// Read the contents of the file as bytes.
	pub fn read_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		use std::{ fs::File, io::Read };
		
		if self.is_dir() {
			Err(format!("Could not read dir \"{}\". Only able to read files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not read file \"{}\". File does not exist.", self.path()).into())
		} else {
			let mut file:File = File::open(self.path())?;
			let mut content:Vec<u8> = Vec::new();
			file.read_to_end(&mut content)?;
			Ok(content)
		}
	}
	
	/// Read a specific range of bytes from the file.
	pub fn read_range(&self, start:u64, end:u64) -> Result<Vec<u8>, Box<dyn Error>> {
		use std::{ fs::File, io::{ Read, Seek, SeekFrom } };

		if self.is_dir() {
			Err(format!("Could not read dir \"{}\". Only able to read files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not read file \"{}\". File does not exist.", self.path()).into())
		} else {
			let mut file:File = File::open(self.path())?;
			let mut buffer:Vec<u8> = vec![0; (end - start) as usize];
			file.seek(SeekFrom::Start(start))?;
			file.read_exact(&mut buffer)?;
			Ok(buffer)
		}
	}



	/* FILE WRITING METHODS */

	/// If the file/dir does not exist, create it.
	pub fn guarantee_exists(&self) -> Result<(), Box<dyn Error>> {
		if !self.exists() {
			self.create()?;
		}
		Ok(())
	}

	/// If the parent dir does not exist, create it.
	pub fn guarantee_parent_dir(&self) -> Result<(), Box<dyn Error>> {
		let parent_dir:FileRef = self.parent_dir()?;
		if !parent_dir.exists() {
			parent_dir.guarantee_parent_dir()?;
			parent_dir.create()?;
		}
		Ok(())
	}

	/// Create the file.
	pub fn create(&self) -> Result<(), Box<dyn Error>> {
		use std::fs::{ File, create_dir };

		let is_dir:bool = self.is_dir();
		if self.exists() {
			Err(format!("Could not create {} \"{}\". {} already exists.", if is_dir { "dir" } else { "file" }, self.path(), if is_dir { "Dir" } else { "File" }).into())
		} else {
			self.guarantee_parent_dir()?;
			if is_dir {
				create_dir(self.path()).map_err(|error| error.into())
			} else {
				File::create(&self.path())?;
				Ok(())
			}
		}
	}

	/// Write a string to the file.
	pub fn write(&self, contents:&str) -> Result<(), Box<dyn Error>> {
		if self.is_dir() {
			Err(format!("Could not write to dir \"{}\". Only able to write to files.", self.path()).into())
		} else {
			self.write_bytes(contents.to_string().as_bytes())
		}
	}

	/// Write bytes to the file.
	pub fn write_bytes(&self, data:&[u8]) -> Result<(), Box<dyn Error>> {
		use std::{ fs::{ File, OpenOptions }, io::Write };
		
		if self.is_dir() {
			Err(format!("Could not write to dir \"{}\". Only able to write to files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not write to file \"{}\". File does not exist.", self.path()).into())
		} else {
			self.guarantee_exists()?;
			let mut file:File = OpenOptions::new().write(true).truncate(true).open(self.path())?;
			file.write_all(data)?;
			Ok(())
		}
	}
	
	/// Read a specific range of bytes from the file.
	pub fn write_bytes_to_range(&self, start:u64, data:&[u8]) -> Result<(), Box<dyn Error>> {
		use std::{ fs::{ File, OpenOptions }, io::{ Write, Seek, SeekFrom } };

		if self.is_dir() {
			Err(format!("Could not write to dir \"{}\". Only able to write to files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not write to file \"{}\". File does not exist.", self.path()).into())
		} else {
			let mut file:File = OpenOptions::new().write(true).open(self.path())?;
			file.seek(SeekFrom::Start(start))?;
			file.write_all(data).map_err(|error| error.into())
		}
	}

	/// Append bytes to the file.
	pub fn append_bytes(&self, data:&[u8]) -> Result<(), Box<dyn Error>> {
		use std::{ fs::{ File, OpenOptions }, io::Write };

		if self.is_dir() {
			Err(format!("Could not append to dir \"{}\". Only able to append to files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not append to file \"{}\". File does not exist.", self.path()).into())
		} else {
			self.guarantee_exists()?;
			let mut file:File = OpenOptions::new().append(true).open(self.path())?;
			file.write_all(data)?;
			Ok(())
		}
	}



	/* FILE MOVING METHODS */

	/// Copy the file to another location. Returns the number of bytes written.
	pub fn copy_to(&self, target:&FileRef) -> Result<u64, Box<dyn Error>> {
		use std::fs::copy;

		if self.is_dir() {
			Err(format!("Could not copy dir \"{}\". Only able to copy files.", self.path()).into())
		} else if !self.exists() {
			Err(format!("Could not copy file \"{}\". File does not exist.", self.path()).into())
		} else {
			target.guarantee_parent_dir()?;
			copy(self.path(), target.path()).map_err(|error| error.into())
		}
	}



	/* FILE REMOVING METHODS */

	/// Delete the file.
	pub fn delete(&self) -> Result<(), Box<dyn Error>> {
		use std::fs::{ remove_dir_all, remove_file };

		if self.is_dir() {
			remove_dir_all(self.path()).map_err(|error| error.into())
		} else {
			remove_file(self.path()).map_err(|error| error.into())
		}
	}



	/* QUICK SCANNER METHODS */

	/// Create a basic scanner on this dir.
	pub fn scanner(&self) -> FileScanner {
		FileScanner::new(self)
	}

	/// Create a file-scanner on this dir that lists all files.
	pub fn list_files(&self) -> FileScanner {
		self.scanner().include_files()
	}
	
	/// Create a file-scanner on this dir that lists all files recursively.
	pub fn list_files_recurse(&self) -> FileScanner {
		self.scanner().include_files().recurse()
	}

	/// Create a file-scanner on this dir that lists all dirs.
	pub fn list_dirs(&self) -> FileScanner {
		self.scanner().include_dirs()
	}

	/// Create a file-scanner on this dir that lists all dirs.
	pub fn list_dirs_recurse(&self) -> FileScanner {
		self.scanner().include_dirs().recurse()
	}
}
impl Add<&str> for FileRef {
	type Output = FileRef;

	fn add(self, rhs:&str) -> Self::Output {
		FileRef::new(&(self.path().to_owned() + rhs))
	}
}
impl AddAssign<&str> for FileRef {
	fn add_assign(&mut self, rhs:&str) {
		*self = FileRef::new(&(self.path().to_owned() + rhs));
	}
}



/* STR INHERITED METHODS */
macro_rules! impl_inherit_str {

	// Case for methods without arguments.
	($fn_name:ident, $output_type:ty) => {
		impl FileRef {
			pub fn $fn_name(&self) -> $output_type {
				self.path().$fn_name()
			}
		}
	};

	// Case for methods with arguments.
	($fn_name:ident, $output_type:ty, ($($arg_name:ident :$arg_type:ty),*)) => {
		impl FileRef {
			pub fn $fn_name(&self, $($arg_name:$arg_type),*) -> $output_type {
				self.path().$fn_name($($arg_name),*)
			}
		}
	};

	// Case for methods returning `FileRef`.
	(ret_self $fn_name:ident) => {
		impl FileRef {
			pub fn $fn_name(&self) -> FileRef {
				FileRef::new(&self.path().$fn_name())
			}
		}
	};

	// Case for methods returning `FileRef` with arguments.
	(ret_self $fn_name:ident, ($($arg_name:ident :$arg_type:ty),*)) => {
		impl FileRef {
			pub fn $fn_name(&self, $($arg_name:$arg_type),*) -> FileRef {
				FileRef::new(&self.path().$fn_name($($arg_name),*))
			}
		}
	};

	// Case for methods returning `Option<FileRef>`.
	(ret_self_opt $fn_name:ident) => {
		impl FileRef {
			pub fn $fn_name(&self) -> Option<FileRef> {
				self.path().$fn_name().map(|path| FileRef::new(path))
			}
		}
	};

	// Case for methods returning `Option<FileRef>` with arguments.
	(ret_self_opt $fn_name:ident, ($($arg_name:ident :$arg_type:ty),*)) => {
		impl FileRef {
			pub fn $fn_name(&self, $($arg_name:$arg_type),*) -> Option<FileRef> {
				self.path().$fn_name($($arg_name),*).map(|path| FileRef::new(path))
			}
		}
	};
}
impl_inherit_str!(len, usize);
impl_inherit_str!(is_empty, bool);
impl_inherit_str!(is_char_boundary, bool, (index:usize));
impl_inherit_str!(contains, bool, (pattern:&str));
impl_inherit_str!(starts_with, bool, (prefix:&str));
impl_inherit_str!(ends_with, bool, (suffix:&str));
impl_inherit_str!(find, Option<usize>, (needle:&str));
impl_inherit_str!(rfind, Option<usize>, (needle:&str));
impl_inherit_str!(split_at, (&str, &str), (mid:usize));
impl_inherit_str!(chars, std::str::Chars<'_>);
impl_inherit_str!(char_indices, std::str::CharIndices<'_>);
impl_inherit_str!(bytes, std::str::Bytes<'_>);
impl_inherit_str!(lines, std::str::Lines<'_>);
impl_inherit_str!(split_whitespace, std::str::SplitWhitespace<'_>);
impl_inherit_str!(split, std::str::Split<'_, char>, (sep:char));
impl_inherit_str!(escape_debug, std::str::EscapeDebug<'_>);
impl_inherit_str!(escape_default, std::str::EscapeDefault<'_>);
impl_inherit_str!(escape_unicode, std::str::EscapeUnicode<'_>);
impl_inherit_str!(splitn, std::str::SplitN<'_, char>, (n:usize, sep:char));
impl_inherit_str!(rsplitn, std::str::RSplitN<'_, char>, (n:usize, sep:char));
impl_inherit_str!(ret_self to_lowercase);
impl_inherit_str!(ret_self to_uppercase);
impl_inherit_str!(ret_self trim);
impl_inherit_str!(ret_self trim_start);
impl_inherit_str!(ret_self trim_start_matches, (pat:&str));
impl_inherit_str!(ret_self trim_end);
impl_inherit_str!(ret_self trim_end_matches, (pat:&str));
impl_inherit_str!(ret_self repeat, (n:usize));
impl_inherit_str!(ret_self replace, (from:&str, to:&str));
impl_inherit_str!(ret_self_opt strip_prefix, (prefix:&str));
impl_inherit_str!(ret_self_opt strip_suffix, (suffix:&str));



#[cfg(test)]
mod tests {
	use unit_test_support::TempFile;
	use super::*;
	


	/* PATH TESTS */
	
	#[test]
	fn test_new() {
		let path:&str = "dir\\file.txt";
		let fs_path:FileRef = FileRef::new(path);
		assert_eq!(fs_path.path(), "dir/file.txt");
	}

	#[test]
	fn test_new_const() {
		const PATH:&str = "static/dir/file.txt";
		let fs_path:FileRef = FileRef::new_const(PATH);
		assert_eq!(fs_path.path(), PATH);
	}

	#[test]
	fn test_path() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		assert_eq!(fs_path.path(), "dir/file.txt");
	}

	#[test]
	fn test_parent_dir() {
		let fs_path:FileRef = FileRef::new("dir/subdir/file.txt");
		let parent:FileRef = fs_path.parent_dir().unwrap();
		assert_eq!(parent.path(), "dir/subdir");
	}

	#[test]
	fn test_parent_dir_root() {
		let fs_path:FileRef = FileRef::new("file.txt");
		assert!(fs_path.parent_dir().is_err());
	}

	#[test]
	fn test_path_nodes() {
		let fs_path:FileRef = FileRef::new("dir/subdir/file.txt");
		let nodes:Vec<&str> = fs_path.path_nodes();
		assert_eq!(nodes, vec!["dir", "subdir", "file.txt"]);
	}

	#[test]
	fn test_last_node() {
		let fs_path:FileRef = FileRef::new("dir/subdir/file.txt");
		assert_eq!(fs_path.last_node(), "file.txt");
	}

	#[test]
	fn test_len() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		assert_eq!(fs_path.len(), 12);
	}

	#[test]
	fn test_is_empty() {
		let fs_path:FileRef = FileRef::new("");
		assert!(fs_path.is_empty());

		let fs_path:FileRef = FileRef::new("not_empty");
		assert!(!fs_path.is_empty());
	}

	#[test]
	fn test_contains() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		assert!(fs_path.contains("file"));
		assert!(!fs_path.contains("no_file"));
	}

	#[test]
	fn test_starts_with() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		assert!(fs_path.starts_with("dir"));
		assert!(!fs_path.starts_with("file"));
	}

	#[test]
	fn test_ends_with() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		assert!(fs_path.ends_with("file.txt"));
		assert!(!fs_path.ends_with("dir"));
	}

	#[test]
	fn test_to_lowercase() {
		let fs_path:FileRef = FileRef::new("DIR/FILE.TXT");
		let lower:FileRef = fs_path.to_lowercase();
		assert_eq!(lower.path(), "dir/file.txt");
	}

	#[test]
	fn test_to_uppercase() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		let upper:FileRef = fs_path.to_uppercase();
		assert_eq!(upper.path(), "DIR/FILE.TXT");
	}

	#[test]
	fn test_trim() {
		let fs_path:FileRef = FileRef::new("   dir/file.txt   ");
		let trimmed:FileRef = fs_path.trim();
		assert_eq!(trimmed.path(), "dir/file.txt");
	}

	#[test]
	fn test_strip_prefix() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		let stripped:Option<FileRef> = fs_path.strip_prefix("dir/");
		assert!(stripped.is_some());
		assert_eq!(stripped.unwrap().path(), "file.txt");
	}

	#[test]
	fn test_strip_suffix() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		let stripped:Option<FileRef> = fs_path.strip_suffix(".txt");
		assert!(stripped.is_some());
		assert_eq!(stripped.unwrap().path(), "dir/file");
	}

	#[test]
	fn test_replace() {
		let fs_path:FileRef = FileRef::new("dir/file.txt");
		let replaced:FileRef = fs_path.replace("file", "document");
		assert_eq!(replaced.path(), "dir/document.txt");
	}

	#[test]
	fn test_repeat() {
		let fs_path:FileRef = FileRef::new("file_");
		let repeated:FileRef = fs_path.repeat(3);
		assert_eq!(repeated.path(), "file_file_file_");
	}



	/* FILE MODIFICATION TESTS */

	#[test]
	fn test_file_creation() {
		let temp_file:TempFile = TempFile::new(Some("txt"));
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());
		assert!(!temp_file_ref.exists());
		temp_file_ref.create().unwrap();
		assert!(temp_file_ref.exists());
	}

	#[test]
	fn test_file_write_and_read() {
		let temp_file:TempFile = TempFile::new(Some("txt"));
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());

		temp_file_ref.create().unwrap();

		let content = "Hello, world!";
		temp_file_ref.write(content).unwrap();

		let read_content = temp_file_ref.read().unwrap();
		assert_eq!(content, read_content);
	}

	#[test]
	fn test_file_write_bytes_and_read_bytes() {
		let temp_file:TempFile = TempFile::new(Some("txt"));
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());

		temp_file_ref.create().unwrap();

		let content = b"Hello, binary world!";
		temp_file_ref.write_bytes(content).unwrap();

		let read_content = temp_file_ref.read_bytes().unwrap();
		assert_eq!(content, read_content.as_slice());
	}

	#[test]
	fn test_append_bytes() {
		let temp_file:TempFile = TempFile::new(Some("txt"));
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());
		
		temp_file_ref.create().unwrap();

		let initial_content = "Hello";
		let append_content = ", world!";
		temp_file_ref.write(initial_content).unwrap();
		temp_file_ref.append_bytes(append_content.as_bytes()).unwrap();

		let read_content = temp_file_ref.read().unwrap();
		assert_eq!(read_content, "Hello, world!");
	}

	#[test]
	fn test_read_range() {
		let temp_file:TempFile = TempFile::new(Some("txt"));
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());

		temp_file_ref.create().unwrap();

		let content = "Hello, world!";
		temp_file_ref.write(content).unwrap();

		let range_content = temp_file_ref.read_range(7, 12).unwrap();
		assert_eq!(std::str::from_utf8(&range_content).unwrap(), "world");
	}

	#[test]
	fn test_write_bytes_to_range() {
		let temp_file:TempFile = TempFile::new(Some("txt"));
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());

		temp_file_ref.create().unwrap();

		let content = "Hello, world!";
		temp_file_ref.write(content).unwrap();

		let replacement = "Rust!";
		temp_file_ref.write_bytes_to_range(7, replacement.as_bytes()).unwrap();

		let read_content = temp_file_ref.read().unwrap();
		assert_eq!(read_content, "Hello, Rust!!");
	}

	#[test]
	fn test_file_deletion() {
		let temp_file:TempFile = TempFile::new(Some("txt"));
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());

		temp_file_ref.create().unwrap();
		assert!(temp_file_ref.exists());

		temp_file_ref.delete().unwrap();
		assert!(!temp_file_ref.exists());
	}

	#[test]
	fn test_file_copy() {
		let temp_file:TempFile = TempFile::new(Some("txt"));
		let temp_file_ref:FileRef = FileRef::new(temp_file.path());
		let source_file_ref = temp_file_ref.clone();
		let target_file_ref = temp_file_ref + "_target.txt";

		source_file_ref.create().unwrap();
		let content = "Copy this content.";
		source_file_ref.write(content).unwrap();

		source_file_ref.copy_to(&target_file_ref).unwrap();
		assert!(target_file_ref.exists());

		let copied_content = target_file_ref.read().unwrap();
		assert_eq!(content, copied_content);

		target_file_ref.delete().unwrap();
	}
}
