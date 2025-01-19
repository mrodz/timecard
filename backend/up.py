"""
Migration script to update local dev table
"""

import os
import shutil
import subprocess
import sys

PORT = 3588
DB_NAME = "postgres"
DB_USER = "postgres"
DB_PASS = "postgres"
ENTITY_EXTRA_DERIVES = ["PartialOrd", "Ord"]
ENTITY_DST = "entity/src/entities"

def main():
    sea = shutil.which("sea-orm-cli")

    my_env = os.environ.copy()
    
    my_env["DATABASE_URL"] = f"postgres://{DB_USER}:{DB_PASS}@localhost:{PORT}/{DB_NAME}"
    
    print("Starting database migration...")
    
    try:
        subprocess.check_call([sea, "migrate", "refresh"], env=my_env, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    except subprocess.CalledProcessError:
        print("Could not migrate database", file=sys.stderr)
        exit(1)
    
    print("Migrated database, starting code generation...")
    
    try:
        subprocess.check_call([sea, "generate", "entity", "-o", ENTITY_DST, "--with-serde", "both", "--model-extra-derives", ','.join(ENTITY_EXTRA_DERIVES)], env=my_env, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    except subprocess.CalledProcessError:
        print("Could not generate code for entities", file=sys.stderr)
        exit(1)
        
    print("Finished code generation\nDone ✅")
if __name__ == "__main__":
    main()


