c64xlarge with 16 cores 36 minutes 12.4 billion before RwLock
c64xlarge with 16 cores 17 minutes 12.4 billion after RwLock



ssh -i ~/linux-laptop.pem ec2-user@18.234.61.46 
scp -i ~/linux-laptop.pem ~/Rust/dfstimizer/src/* ec2-user@18.234.61.46:/home/ec2-user/dfstimizer/src
scp -i ~/linux-laptop.pem ~/Rust/dfstimizer/* ec2-user@18.234.61.46:/home/ec2-user/dfstimizer