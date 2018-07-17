import kubos_test
config_location = "../common/overlay/home/system/etc/config.toml"

if __name__ == '__main__':
    test = kubos_test.IntegrationTest(config_location)
    print "Services Test"
    print "#############\n"
    test.test_services()
