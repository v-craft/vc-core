pub trait DynamicBundle: Sized {
    type Effect;

    // TODO
}

pub unsafe trait Bundle: DynamicBundle + Send + Sync + 'static {
    // TODO
}

pub unsafe trait BundleFromComponents {
    // TODO
}

pub trait NoEffectBundle {}
