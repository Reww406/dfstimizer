c64xlarge with 16 cores 36 minutes 12.4 billion before RwLock
c64xlarge with 16 cores 17 minutes 12.4 billion after RwLock
c7 8xlarge with 32 cores 102 miuntes 122B
c7 8xlarge with 32 cores 288 minutes 168B
c7 8xlarge with 32 cores 588 minutes 483B
c7 8xlarget with 32 cores 179 miuntes 169B

ssh -i ~/linux-laptop.pem ec2-user@54.210.216.173
scp -i ~/linux-laptop.pem ~/Rust/dfstimizer/src/* ec2-user@54.210.216.173:/home/ec2-user/dfstimizer/src
scp -i ~/linux-laptop.pem ~/Rust/dfstimizer/* ec2-user@54.210.216.173:/home/ec2-user/dfstimizer

ssh -i ~/linux-laptop.pem ec2-user@54.210.216.173 "cat ~/dfstimizer/lineups/lineups-3-thu.txt" > ~/lineups/lineups-3-sun-1.txt