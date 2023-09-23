c64xlarge with 16 cores 36 minutes 12.4 billion before RwLock
c64xlarge with 16 cores 17 minutes 12.4 billion after RwLock
c7 8xlarge with 32 cores 102 miuntes 122B
c7 8xlarge with 32 cores 288 minutes 168B

ssh -i ~/linux-laptop.pem ec2-user@52.90.126.26
scp -i ~/linux-laptop.pem ~/Rust/dfstimizer/src/* ec2-user@52.90.126.26:/home/ec2-user/dfstimizer/src
scp -i ~/linux-laptop.pem ~/Rust/dfstimizer/* ec2-user@52.90.126.26:/home/ec2-user/dfstimizer