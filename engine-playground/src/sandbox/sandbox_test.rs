use super::*;

#[tokio::test]
async fn test_simple() {
    let status = run_sandboxed("echo AAAAA", ".", Duration::from_millis(500))
        .await
        .unwrap();
    assert_eq!(0, status);
}

// TODO test all the possible escapes:
/*
id
pwd
ls -l /
ls -l .
ip link
ls -l /proc/self/fd
ulimit -a
sleep 5
touch test.txt
mount
ps -aux
 */
