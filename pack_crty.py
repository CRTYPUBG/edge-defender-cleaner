import zipfile
import os
import glob

print("Packing resources into res.crty...")

with zipfile.ZipFile('res.crty', 'w', zipfile.ZIP_DEFLATED) as z:
    for dir_name in ['Remove_Defender', 'Remove_SecurityComp', 'Remove-MS-Edge-main']:
        if os.path.exists(dir_name):
            for root, _, files in os.walk(dir_name):
                for f in files:
                    file_path = os.path.join(root, f)
                    arcname = os.path.relpath(file_path, ".")
                    print(f"Adding {arcname}")
                    z.write(file_path, arcname)
    
    for ext in ['*.bat', '*.ps1', '*.exe', '*.old']:
        for f in glob.glob(ext):
            f_lower = f.lower()
            if 'edgedefendercleaner' in f_lower or f_lower == 'setup.exe':
                continue
            print(f"Adding {f}")
            z.write(f, f)

print("Done creating res.crty")
