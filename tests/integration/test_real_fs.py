import os
import random
import string
from tempfile import TemporaryDirectory
import subprocess
import time

def test_real_fs():
    # Create a temporary directory to serve
    with TemporaryDirectory() as temp_dir:
        # Create some test files and directories
        os.mkdir(os.path.join(temp_dir, "test_dir"))
        with open(os.path.join(temp_dir, "test_file.txt"), "w") as f:
            f.write("This is a test file")
        
        with open(os.path.join(temp_dir, "test_dir", "nested_file.txt"), "w") as f:
            f.write("This is a nested file")
        
        # Start the NFS server with the temporary directory
        bold_mem = ["cargo", "run", "-p", "bold-mem", "--bin", "bold-real-fs", "--"]
        args = [temp_dir]
        proc = subprocess.Popen(bold_mem + args)
        time.sleep(1)  # Give the server time to start
        assert proc.poll() is None  # Check that the process is still running
        
        # Mount the NFS share
        mount_point = "/tmp/test_nfs"
        os.makedirs(mount_point, exist_ok=True)
        
        try:
            # Mount the NFS share
            mount_cmd = ["sudo", "mount.nfs4", "-n", "-o", "fg,soft,sec=none,vers=4.0,port=11112", "127.0.0.1:/", mount_point]
            mount_result = subprocess.run(mount_cmd)
            assert mount_result.returncode == 0
            
            # Check that the files are accessible
            dirs = os.listdir(mount_point)
            assert "test_dir" in dirs
            assert "test_file.txt" in dirs
            
            # Check file contents
            with open(os.path.join(mount_point, "test_file.txt"), "r") as f:
                content = f.read()
                assert content == "This is a test file"
            
            # Check nested file
            with open(os.path.join(mount_point, "test_dir", "nested_file.txt"), "r") as f:
                content = f.read()
                assert content == "This is a nested file"
            
            # Create a new file through NFS
            with open(os.path.join(mount_point, "new_file.txt"), "w") as f:
                f.write("This is a new file created through NFS")
            
            # Check that the new file exists in the original directory
            assert os.path.exists(os.path.join(temp_dir, "new_file.txt"))
            with open(os.path.join(temp_dir, "new_file.txt"), "r") as f:
                content = f.read()
                assert content == "This is a new file created through NFS"
                
        finally:
            # Unmount the NFS share
            subprocess.run(["sudo", "umount", "-f", mount_point])
            # Stop the NFS server
            proc.kill()