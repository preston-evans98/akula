use ethereum::Header;

#[derive(Copy, Clone)]
pub enum HeaderSliceStatus {
    Empty,
    Downloaded,
    Verified,
    Saved,
}

pub struct HeaderSlices {
    headers: Vec<Header>,
    statuses: Vec<HeaderSliceStatus>,
}

const HEADER_SLICE_SIZE: usize = 192;

impl HeaderSlices {
    pub fn new(capacity: usize) -> Self {
        let max_slices = capacity / std::mem::size_of::<Header>() / HEADER_SLICE_SIZE;
        let max_headers = max_slices * HEADER_SLICE_SIZE;

        let headers = Vec::<Header>::with_capacity(max_headers);
        let statuses = vec![HeaderSliceStatus::Empty; max_slices];

        Self { headers, statuses }
    }

    pub fn clone_statuses(&self) -> Vec<HeaderSliceStatus> {
        self.statuses.clone()
    }
}
