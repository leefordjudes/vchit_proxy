# Oracle basic client 23.26 setup in ubuntu 24.04

download basic package zip from 

https://www.oracle.com/in/database/technologies/instant-client/linux-x86-64-downloads.html

- recommended version is 19.32, 

https://www.oracle.com/in/database/technologies/instant-client/linux-x86-64-downloads.html#version-1932000 <br/>
https://download.oracle.com/otn_software/linux/instantclient/1932000/instantclient-basic-linux.x64-19.32.0.0.0dbru.zip

- but i used 23.26 latest - tried and worked

https://download.oracle.com/otn_software/linux/instantclient/2326200v2/instantclient-basic-linux.x64-23.26.2.0.0.zip


Move and Check the Directory Path
    sudo mkdir -p /opt/oracle/

Unzip the downloaded zip file into /opt/oracle/    
    sudo unzip instantclient-basic-linux.x64-23.26.2.0.0.zip -d /opt/oracle/

Verify that the libclntsh.so file exists
    ls /opt/oracle/instantclient_23_26/libclntsh.so*

Register the Library Path
    echo "/opt/oracle/instantclient_23_26" | sudo tee /etc/ld.so.conf.d/oracle-instantclient.conf

Clear and Rebuild Runtime Caches
    sudo ldconfig

Update Your Shell Profile
    nano ~/.bashrc
    
    echo 'export ORACLE_HOME=/opt/oracle/instantclient_23_26' >> ~/.bashrc
    echo 'export LD_LIBRARY_PATH=$ORACLE_HOME:$LD_LIBRARY_PATH' >> ~/.bashrc

Save, close, and reload the profile:
    source ~/.bashrc    

Install the Asynchronous I/O Package
    sudo apt update && sudo apt install -y libaio1

Create a Symlink Fix (If Using Ubuntu 24.04+)
    # Locate and link the modern file variant to the legacy name Oracle expects
    sudo ln -s /usr/lib/x86_64-linux-gnu/libaio.so.1t64 /usr/lib/x86_64-linux-gnu/libaio.so.1


Clear and Rebuild Runtime Caches
    sudo ldconfig


Open the CSF Configuration File
    sudo nano /etc/csf/csf.conf

Add Port 1521 to Outbound Lists
    TCP_OUT = "20,21,22,25,53,80,110,443,1521"

Save and Close

Restart the CSF Daemon
    sudo csf -r
    
Verify the Connection    
    nc -zv velcloud.in 1521

## Test Oracle Connection

- main.rs
```rs
use oracle::Connection;

fn main() {
    // Replace "username" and "password" with your actual database credentials
    let user = "username";
    let password = "password";
    let connect_string = "oracle connection string";

    println!("Attempting to connect to Oracle at {}...", connect_string);

    match Connection::connect(user, password, connect_string) {
        Ok(conn) => {
            println!(" Successfully connected to the database!");

            // Run a quick test query to check database responsiveness
            match conn.query_row_as::<(String,)>("SELECT TO_CHAR(SYSDATE, 'YYYY-MM-DD HH24:MI:SS') FROM DUAL", &[]) {
                Ok((sysdate,)) => {
                    println!("Oracle Server Database Time: {}", sysdate);
                }
                Err(e) => {
                    println!("❌ Connected, but failed to execute query: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Database connection handshake failed!");
            println!("Error details: {}", e);
            println!("\nTroubleshooting tips if credentials are right:");
            println!("- If it says 'ORA-28040: No matching authentication protocol', your Instant Client 23 is too new for this old XE server. You will need to downgrade to Instant Client 19c.");
        }
    }
}
```

- Cargo.toml
```toml
[dependencies]
oracle = "0.6.3"
```