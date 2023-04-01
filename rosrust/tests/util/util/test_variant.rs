/// Each integration test has a unique port for its `roscore` instance.
///
/// For any new integration test, add an option to the enum and use it.
#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum TestVariant {
    Benchmark = 0,
    CanLogOnce,
    CanReadLogFromRosoutForMultiple,
    CanReadLogFromRosoutForRoscpp,
    CanReadLogFromRosoutForRospy,
    CanReadLogFromRosoutForRosrust,
    CanReadLogFromRosout,
    CanThrottleIdenticalLogs,
    CanThrottleLogs,
    ClientToInlineService,
    ClientToRoscppService,
    ClientToRospyServiceReconnection,
    ClientToRospyService,
    ClientToRosrustService,
    DeriveArrayTest,
    DynamicMsg,
    MsgToAndFromValue,
    PublisherToInlineSubscriber,
    PublisherToMultipleSubscribers,
    PublisherToRelayedSubscriber,
    PublisherToRoscppSubscriber,
    PublisherToRospySubscriber,
    PublisherToRosrustSubscriber,
    ReservedKeywordsTest,
    RosparamGetSet,
    ServiceToRoscppClient,
    ServiceToRospyClient,
    ServiceToRosrustClient,
    ServiceToRosserviceClient,
    SubscriberToMultiplePublishers,
    SubscriberToRoscppPublisher,
    SubscriberToRospyPublisher,
    SubscriberToRosrustPublisher,
    SubscriberToRostopicPublisher,
    WaitForService,
}

impl TestVariant {
    pub fn port(self) -> u32 {
        self as u32 + 11501
    }
}
