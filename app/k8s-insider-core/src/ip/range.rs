use std::net::Ipv4Addr;

use ipnet::{IpAdd, Ipv4Net};
use num_traits::{AsPrimitive, FromPrimitive, Unsigned};
use rand::{rngs::OsRng, RngCore};

#[derive(Debug)]
pub struct UniqueRandomWrappingHostsIpIterator<IP, IPSize> {
    first_address: IP,
    last_address: IP,
    subnet_count: IPSize,
    previous_offset: IPSize,
}

impl<IP, IPSize> UniqueRandomWrappingHostsIpIterator<IP, IPSize>
where
    IP: Copy + IpAdd<IPSize, Output = IP> + PartialEq<IP>,
    IPSize: Unsigned + FromPrimitive + AsPrimitive<u128>,
{
    pub fn get(&mut self) -> IP {
        let subnet_count = self.subnet_count.as_();
        let previous_offset = self.previous_offset.as_();

        // https://en.wikipedia.org/wiki/Linear_congruential_generator
        let next_offset = (previous_offset * 5 + 1) % subnet_count;
        let next_offset = IPSize::from_u128(next_offset).unwrap(); // assuming above equasion is correct, this shouldn't panic

        let address = self.first_address.saturating_add(next_offset);

        self.previous_offset = next_offset;

        if subnet_count > 2 && (address == self.first_address || address == self.last_address) {
            return self.get();
        }

        address
    }
}

impl<IP, IPSize> UniqueRandomWrappingHostsIpIterator<IP, IPSize>
where
    IPSize: AsPrimitive<u128>,
{
    pub fn address_count(&self) -> u128 {
        self.subnet_count.as_()
    }
}

impl UniqueRandomWrappingHostsIpIterator<Ipv4Addr, u32> {
    pub fn new(subnet: Ipv4Net) -> Self {
        let subnet_count = 1 << (32 - subnet.prefix_len());
        let first_address = subnet.network();
        let last_address = subnet.broadcast();
        Self {
            first_address,
            last_address,
            subnet_count,
            previous_offset: OsRng.next_u32() % subnet_count,
        }
    }
}

impl<IP, IPSize> Iterator for UniqueRandomWrappingHostsIpIterator<IP, IPSize>
where
    IP: Copy + IpAdd<IPSize, Output = IP> + PartialEq<IP>,
    IPSize: Unsigned + FromPrimitive + AsPrimitive<u128>,
{
    type Item = IP;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.get())
    }
}

#[cfg(test)]
mod tests {
    use ipnet::Ipv4Net;

    use super::UniqueRandomWrappingHostsIpIterator;

    #[test]
    fn unique_random_subnet_iterator_iterates_through_all_addresses_once() {
        iterates_through_all_addresses_once("10.13.2.4/30");
        iterates_through_all_addresses_once("123.43.3.1/24");
        iterates_through_all_addresses_once("192.168.0.0/29");
        iterates_through_all_addresses_once("2.18.0.0/16");
        iterates_through_all_addresses_once("50.168.56.0/29");
        iterates_through_all_addresses_once("192.43.1.0/14");
    }

    fn iterates_through_all_addresses_once(net_raw: &str) {
        let net: Ipv4Net = net_raw.parse().unwrap();
        let range = net.hosts().collect::<Vec<_>>();
        let mut iteration_result = UniqueRandomWrappingHostsIpIterator::new(net)
            .take(range.len())
            .collect::<Vec<_>>();

        iteration_result.sort_unstable();

        assert_eq!(range, iteration_result);
    }
}
