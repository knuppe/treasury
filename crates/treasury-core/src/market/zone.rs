//! Fusos horários das bolsas, numa tabela nossa (no lugar da base IANA inteira): o desvio
//! padrão de cada fuso e, quando há horário de verão, a regra em vigor. Cobre os fusos do
//! catálogo de bolsas e os dos principais centros financeiros, pelo nome IANA que o
//! catálogo e o Yahoo usam (`America/Sao_Paulo`, `Europe/London`…).
//!
//! As regras são as atuais e valem para qualquer ano: um instante antes de uma mudança de
//! regra (o Brasil teve horário de verão até 2019, o México até 2022) sai com a regra de
//! hoje. Basta para o uso do núcleo, que é o pregão de hoje e as cotações intradiárias
//! recentes. Quando um país muda a regra, muda aqui (e o teste da transição do ano).

use std::fmt;

use chrono::{
    Datelike, Duration, FixedOffset, MappedLocalTime, NaiveDate, NaiveDateTime, NaiveTime, Offset,
    TimeZone, Weekday,
};

const H: i32 = 3600;

/// Regra de horário de verão (adiantamento de uma hora).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Rule {
    /// Sem horário de verão.
    None,
    /// União Europeia e vizinhos: último domingo de março às 01:00 UTC até o último
    /// domingo de outubro às 01:00 UTC.
    Eu,
    /// Estados Unidos e Canadá: segundo domingo de março às 02:00 locais até o primeiro
    /// domingo de novembro às 02:00 locais.
    Us,
    /// Chile: domingo a partir do dia 2 de setembro às 04:00 UTC até o domingo a partir do
    /// dia 2 de abril às 03:00 UTC.
    Chile,
    /// Sudeste da Austrália: primeiro domingo de outubro até o primeiro domingo de abril,
    /// às 02:00 da hora padrão.
    Au,
    /// Nova Zelândia: último domingo de setembro até o primeiro domingo de abril, às 02:00
    /// da hora padrão.
    Nz,
    /// Israel: sexta a partir do dia 23 de março às 02:00 até o último domingo de outubro
    /// às 02:00 locais.
    Israel,
    /// Egito: última sexta de abril às 00:00 até a última quinta de outubro às 24:00 locais.
    Egypt,
    /// Líbano: último domingo de março às 00:00 até o último domingo de outubro às 00:00
    /// locais.
    Lebanon,
}

/// Qual dia do mês.
#[derive(Clone, Copy)]
enum Day {
    /// O n-ésimo dia da semana do mês (1 = o primeiro).
    Nth(u8, Weekday),
    /// O último dia da semana do mês.
    Last(Weekday),
    /// O primeiro dia da semana a partir do dia do mês.
    From(u32, Weekday),
}

/// Em que relógio está a hora da mudança.
#[derive(Clone, Copy)]
enum Clock {
    Utc,
    /// Hora padrão do fuso.
    Standard,
    /// Hora local em vigor antes da mudança.
    Wall,
}

#[derive(Clone, Copy)]
struct Change {
    month: u32,
    day: Day,
    /// Segundos depois da meia-noite (pode ser 24:00).
    at: i32,
    clock: Clock,
}

const fn change(month: u32, day: Day, hour: i32, clock: Clock) -> Change {
    Change {
        month,
        day,
        at: hour * H,
        clock,
    }
}

impl Rule {
    /// Início e fim do horário de verão.
    fn changes(self) -> Option<(Change, Change)> {
        use Clock::*;
        use Day::*;
        use Weekday::*;
        Some(match self {
            Rule::None => return None,
            Rule::Eu => (change(3, Last(Sun), 1, Utc), change(10, Last(Sun), 1, Utc)),
            Rule::Us => (
                change(3, Nth(2, Sun), 2, Wall),
                change(11, Nth(1, Sun), 2, Wall),
            ),
            Rule::Chile => (
                change(9, From(2, Sun), 4, Utc),
                change(4, From(2, Sun), 3, Utc),
            ),
            Rule::Au => (
                change(10, Nth(1, Sun), 2, Standard),
                change(4, Nth(1, Sun), 2, Standard),
            ),
            Rule::Nz => (
                change(9, Last(Sun), 2, Standard),
                change(4, Nth(1, Sun), 2, Standard),
            ),
            Rule::Israel => (
                change(3, From(23, Fri), 2, Wall),
                change(10, Last(Sun), 2, Wall),
            ),
            Rule::Egypt => (
                change(4, Last(Fri), 0, Wall),
                change(10, Last(Thu), 24, Wall),
            ),
            Rule::Lebanon => (
                change(3, Last(Sun), 0, Wall),
                change(10, Last(Sun), 0, Wall),
            ),
        })
    }
}

/// Um fuso da tabela. Implementa [`TimeZone`]: funciona com `with_timezone` e
/// `from_local_datetime` como qualquer fuso do chrono.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Zone {
    name: &'static str,
    /// Desvio da hora padrão, em segundos a leste de UTC.
    standard: i32,
    rule: Rule,
}

impl fmt::Debug for Zone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

impl fmt::Display for Zone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

/// Desvio de um instante num [`Zone`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZoneOffset {
    zone: Zone,
    offset: FixedOffset,
}

impl Offset for ZoneOffset {
    fn fix(&self) -> FixedOffset {
        self.offset
    }
}

impl fmt::Display for ZoneOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.offset, f)
    }
}

const fn z(name: &'static str, minutes: i32, rule: Rule) -> Zone {
    Zone {
        name,
        standard: minutes * 60,
        rule,
    }
}

/// Os fusos conhecidos, com o desvio padrão em minutos. Nomes antigos que a base IANA
/// ainda aceita (`Asia/Calcutta`, `Europe/Kiev`, `US/Eastern`…) entram como sinônimos.
#[rustfmt::skip]
const ZONES: &[Zone] = &[
    z("UTC", 0, Rule::None),
    z("Etc/UTC", 0, Rule::None),
    z("GMT", 0, Rule::None),
    z("Etc/GMT", 0, Rule::None),
    // América do Sul
    z("America/Sao_Paulo", -180, Rule::None),
    z("America/Fortaleza", -180, Rule::None),
    z("America/Recife", -180, Rule::None),
    z("America/Bahia", -180, Rule::None),
    z("America/Belem", -180, Rule::None),
    z("America/Maceio", -180, Rule::None),
    z("America/Araguaina", -180, Rule::None),
    z("America/Santarem", -180, Rule::None),
    z("America/Noronha", -120, Rule::None),
    z("America/Manaus", -240, Rule::None),
    z("America/Cuiaba", -240, Rule::None),
    z("America/Campo_Grande", -240, Rule::None),
    z("America/Porto_Velho", -240, Rule::None),
    z("America/Boa_Vista", -240, Rule::None),
    z("America/Rio_Branco", -300, Rule::None),
    z("America/Eirunepe", -300, Rule::None),
    z("America/Argentina/Buenos_Aires", -180, Rule::None),
    z("America/Buenos_Aires", -180, Rule::None),
    z("America/Argentina/Cordoba", -180, Rule::None),
    z("America/Montevideo", -180, Rule::None),
    z("America/Asuncion", -180, Rule::None),
    z("America/La_Paz", -240, Rule::None),
    z("America/Lima", -300, Rule::None),
    z("America/Bogota", -300, Rule::None),
    z("America/Guayaquil", -300, Rule::None),
    z("America/Caracas", -240, Rule::None),
    z("America/Santiago", -240, Rule::Chile),
    // América Central, Caribe e México
    z("America/Panama", -300, Rule::None),
    z("America/Mexico_City", -360, Rule::None),
    z("America/Monterrey", -360, Rule::None),
    z("America/Cancun", -300, Rule::None),
    z("America/Costa_Rica", -360, Rule::None),
    z("America/Guatemala", -360, Rule::None),
    z("America/El_Salvador", -360, Rule::None),
    z("America/Tegucigalpa", -360, Rule::None),
    z("America/Managua", -360, Rule::None),
    z("America/Jamaica", -300, Rule::None),
    z("America/Puerto_Rico", -240, Rule::None),
    z("America/Santo_Domingo", -240, Rule::None),
    z("America/Port_of_Spain", -240, Rule::None),
    z("America/Barbados", -240, Rule::None),
    z("America/Nassau", -300, Rule::Us),
    z("Atlantic/Bermuda", -240, Rule::Us),
    // América do Norte
    z("America/New_York", -300, Rule::Us),
    z("America/Detroit", -300, Rule::Us),
    z("America/Chicago", -360, Rule::Us),
    z("America/Denver", -420, Rule::Us),
    z("America/Phoenix", -420, Rule::None),
    z("America/Los_Angeles", -480, Rule::Us),
    z("America/Anchorage", -540, Rule::Us),
    z("Pacific/Honolulu", -600, Rule::None),
    z("US/Eastern", -300, Rule::Us),
    z("US/Central", -360, Rule::Us),
    z("US/Mountain", -420, Rule::Us),
    z("US/Pacific", -480, Rule::Us),
    z("America/Toronto", -300, Rule::Us),
    z("America/Montreal", -300, Rule::Us),
    z("America/Halifax", -240, Rule::Us),
    z("America/St_Johns", -210, Rule::Us),
    z("America/Winnipeg", -360, Rule::Us),
    z("America/Regina", -360, Rule::None),
    z("America/Edmonton", -420, Rule::Us),
    z("America/Vancouver", -480, Rule::Us),
    // Europa
    z("Europe/London", 0, Rule::Eu),
    z("Europe/Dublin", 0, Rule::Eu),
    z("Europe/Lisbon", 0, Rule::Eu),
    z("Europe/Guernsey", 0, Rule::Eu),
    z("Europe/Jersey", 0, Rule::Eu),
    z("Europe/Isle_of_Man", 0, Rule::Eu),
    z("Atlantic/Madeira", 0, Rule::Eu),
    z("Atlantic/Canary", 0, Rule::Eu),
    z("Atlantic/Faroe", 0, Rule::Eu),
    z("Atlantic/Azores", -60, Rule::Eu),
    z("Atlantic/Reykjavik", 0, Rule::None),
    z("Europe/Madrid", 60, Rule::Eu),
    z("Europe/Paris", 60, Rule::Eu),
    z("Europe/Berlin", 60, Rule::Eu),
    z("Europe/Amsterdam", 60, Rule::Eu),
    z("Europe/Brussels", 60, Rule::Eu),
    z("Europe/Luxembourg", 60, Rule::Eu),
    z("Europe/Zurich", 60, Rule::Eu),
    z("Europe/Vaduz", 60, Rule::Eu),
    z("Europe/Vienna", 60, Rule::Eu),
    z("Europe/Rome", 60, Rule::Eu),
    z("Europe/Malta", 60, Rule::Eu),
    z("Europe/Monaco", 60, Rule::Eu),
    z("Europe/Andorra", 60, Rule::Eu),
    z("Europe/Gibraltar", 60, Rule::Eu),
    z("Europe/Stockholm", 60, Rule::Eu),
    z("Europe/Oslo", 60, Rule::Eu),
    z("Europe/Copenhagen", 60, Rule::Eu),
    z("Europe/Warsaw", 60, Rule::Eu),
    z("Europe/Prague", 60, Rule::Eu),
    z("Europe/Bratislava", 60, Rule::Eu),
    z("Europe/Budapest", 60, Rule::Eu),
    z("Europe/Ljubljana", 60, Rule::Eu),
    z("Europe/Zagreb", 60, Rule::Eu),
    z("Europe/Belgrade", 60, Rule::Eu),
    z("Europe/Sarajevo", 60, Rule::Eu),
    z("Europe/Skopje", 60, Rule::Eu),
    z("Europe/Podgorica", 60, Rule::Eu),
    z("Europe/Tirane", 60, Rule::Eu),
    z("Europe/Helsinki", 120, Rule::Eu),
    z("Europe/Athens", 120, Rule::Eu),
    z("Europe/Bucharest", 120, Rule::Eu),
    z("Europe/Sofia", 120, Rule::Eu),
    z("Europe/Riga", 120, Rule::Eu),
    z("Europe/Tallinn", 120, Rule::Eu),
    z("Europe/Vilnius", 120, Rule::Eu),
    z("Europe/Kyiv", 120, Rule::Eu),
    z("Europe/Kiev", 120, Rule::Eu),
    z("Asia/Nicosia", 120, Rule::Eu),
    z("Europe/Nicosia", 120, Rule::Eu),
    z("Europe/Istanbul", 180, Rule::None),
    z("Europe/Moscow", 180, Rule::None),
    z("Europe/Minsk", 180, Rule::None),
    // África
    z("Africa/Johannesburg", 120, Rule::None),
    z("Africa/Windhoek", 120, Rule::None),
    z("Africa/Gaborone", 120, Rule::None),
    z("Africa/Harare", 120, Rule::None),
    z("Africa/Lusaka", 120, Rule::None),
    z("Africa/Maputo", 120, Rule::None),
    z("Africa/Kigali", 120, Rule::None),
    z("Africa/Cairo", 120, Rule::Egypt),
    z("Africa/Lagos", 60, Rule::None),
    z("Africa/Tunis", 60, Rule::None),
    z("Africa/Algiers", 60, Rule::None),
    z("Africa/Accra", 0, Rule::None),
    z("Africa/Abidjan", 0, Rule::None),
    z("Africa/Nairobi", 180, Rule::None),
    z("Africa/Kampala", 180, Rule::None),
    z("Africa/Dar_es_Salaam", 180, Rule::None),
    z("Africa/Addis_Ababa", 180, Rule::None),
    z("Indian/Mauritius", 240, Rule::None),
    // Oriente Médio e Ásia
    z("Asia/Jerusalem", 120, Rule::Israel),
    z("Asia/Tel_Aviv", 120, Rule::Israel),
    z("Asia/Beirut", 120, Rule::Lebanon),
    z("Asia/Amman", 180, Rule::None),
    z("Asia/Damascus", 180, Rule::None),
    z("Asia/Riyadh", 180, Rule::None),
    z("Asia/Qatar", 180, Rule::None),
    z("Asia/Kuwait", 180, Rule::None),
    z("Asia/Bahrain", 180, Rule::None),
    z("Asia/Baghdad", 180, Rule::None),
    z("Asia/Tehran", 210, Rule::None),
    z("Asia/Dubai", 240, Rule::None),
    z("Asia/Muscat", 240, Rule::None),
    z("Asia/Baku", 240, Rule::None),
    z("Asia/Tbilisi", 240, Rule::None),
    z("Asia/Yerevan", 240, Rule::None),
    z("Asia/Karachi", 300, Rule::None),
    z("Asia/Tashkent", 300, Rule::None),
    z("Asia/Almaty", 300, Rule::None),
    z("Asia/Kolkata", 330, Rule::None),
    z("Asia/Calcutta", 330, Rule::None),
    z("Asia/Colombo", 330, Rule::None),
    z("Asia/Kathmandu", 345, Rule::None),
    z("Asia/Dhaka", 360, Rule::None),
    z("Asia/Yangon", 390, Rule::None),
    z("Asia/Bangkok", 420, Rule::None),
    z("Asia/Ho_Chi_Minh", 420, Rule::None),
    z("Asia/Saigon", 420, Rule::None),
    z("Asia/Phnom_Penh", 420, Rule::None),
    z("Asia/Jakarta", 420, Rule::None),
    z("Asia/Makassar", 480, Rule::None),
    z("Asia/Kuala_Lumpur", 480, Rule::None),
    z("Asia/Singapore", 480, Rule::None),
    z("Asia/Manila", 480, Rule::None),
    z("Asia/Hong_Kong", 480, Rule::None),
    z("Asia/Shanghai", 480, Rule::None),
    z("Asia/Taipei", 480, Rule::None),
    z("Asia/Ulaanbaatar", 480, Rule::None),
    z("Asia/Seoul", 540, Rule::None),
    z("Asia/Tokyo", 540, Rule::None),
    // Oceania
    z("Australia/Perth", 480, Rule::None),
    z("Australia/Darwin", 570, Rule::None),
    z("Australia/Adelaide", 570, Rule::Au),
    z("Australia/Brisbane", 600, Rule::None),
    z("Australia/Sydney", 600, Rule::Au),
    z("Australia/Canberra", 600, Rule::Au),
    z("Australia/Melbourne", 600, Rule::Au),
    z("Australia/Hobart", 600, Rule::Au),
    z("Pacific/Guam", 600, Rule::None),
    z("Pacific/Port_Moresby", 600, Rule::None),
    z("Pacific/Auckland", 720, Rule::Nz),
    z("Pacific/Fiji", 720, Rule::None),
];

impl std::str::FromStr for Zone {
    type Err = UnknownZone;

    /// Pelo nome IANA exato (maiúsculas contam, como na base IANA).
    fn from_str(s: &str) -> Result<Self, UnknownZone> {
        ZONES
            .iter()
            .find(|z| z.name == s)
            .copied()
            .ok_or(UnknownZone)
    }
}

/// Nomes dos fusos conhecidos, na ordem da tabela.
pub fn names() -> impl Iterator<Item = &'static str> {
    ZONES.iter().map(|z| z.name)
}

/// O nome não está na tabela.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownZone;

impl Zone {
    pub fn name(&self) -> &'static str {
        self.name
    }

    fn fixed(&self, secs: i32) -> ZoneOffset {
        ZoneOffset {
            zone: *self,
            offset: FixedOffset::east_opt(secs).expect("desvio da tabela dentro de ±24h"),
        }
    }

    /// Instante UTC de uma mudança no ano. `before` é o desvio em vigor antes dela.
    fn change_utc(&self, year: i32, c: Change, before: i32) -> NaiveDateTime {
        let midnight = day_of(year, c.month, c.day).and_time(NaiveTime::MIN);
        let shift = match c.clock {
            Clock::Utc => 0,
            Clock::Standard => self.standard,
            Clock::Wall => before,
        };
        midnight + Duration::seconds(i64::from(c.at - shift))
    }

    /// Desvio em segundos num instante UTC.
    fn offset_at(&self, utc: &NaiveDateTime) -> i32 {
        let Some((start, end)) = self.rule.changes() else {
            return self.standard;
        };
        let summer = self.standard + H;
        let y = utc.year();
        let from = self.change_utc(y, start, self.standard);
        let to = self.change_utc(y, end, summer);
        let dst = if start.month < end.month {
            from <= *utc && *utc < to
        } else {
            // Hemisfério sul: o verão atravessa a virada do ano.
            *utc < to || from <= *utc
        };
        if dst { summer } else { self.standard }
    }
}

/// Dia do mês pela regra.
fn day_of(year: i32, month: u32, day: Day) -> NaiveDate {
    match day {
        Day::Nth(n, wd) => {
            NaiveDate::from_weekday_of_month_opt(year, month, wd, n).expect("dia da regra")
        }
        Day::Last(wd) => {
            let next = if month == 12 {
                NaiveDate::from_ymd_opt(year + 1, 1, 1)
            } else {
                NaiveDate::from_ymd_opt(year, month + 1, 1)
            }
            .expect("mês válido");
            let mut d = next.pred_opt().expect("dia anterior");
            while d.weekday() != wd {
                d = d.pred_opt().expect("dia anterior");
            }
            d
        }
        Day::From(first, wd) => {
            let mut d = NaiveDate::from_ymd_opt(year, month, first).expect("dia da regra");
            while d.weekday() != wd {
                d = d.succ_opt().expect("dia seguinte");
            }
            d
        }
    }
}

impl TimeZone for Zone {
    type Offset = ZoneOffset;

    fn from_offset(offset: &ZoneOffset) -> Self {
        offset.zone
    }

    fn offset_from_local_date(&self, local: &NaiveDate) -> MappedLocalTime<ZoneOffset> {
        self.offset_from_local_datetime(&local.and_time(NaiveTime::MIN))
    }

    fn offset_from_local_datetime(&self, local: &NaiveDateTime) -> MappedLocalTime<ZoneOffset> {
        if self.rule == Rule::None {
            return MappedLocalTime::Single(self.fixed(self.standard));
        }
        // Um desvio serve quando a hora local menos ele cai num instante com esse desvio.
        // Na hora que pula (início do verão) nenhum serve; na que se repete, os dois.
        let fits = |secs: i32| {
            let utc = *local - Duration::seconds(i64::from(secs));
            self.offset_at(&utc) == secs
        };
        let (summer, standard) = (self.standard + H, self.standard);
        match (fits(summer), fits(standard)) {
            (true, true) => MappedLocalTime::Ambiguous(self.fixed(summer), self.fixed(standard)),
            (true, false) => MappedLocalTime::Single(self.fixed(summer)),
            (false, true) => MappedLocalTime::Single(self.fixed(standard)),
            (false, false) => MappedLocalTime::None,
        }
    }

    fn offset_from_utc_date(&self, utc: &NaiveDate) -> ZoneOffset {
        self.offset_from_utc_datetime(&utc.and_time(NaiveTime::MIN))
    }

    fn offset_from_utc_datetime(&self, utc: &NaiveDateTime) -> ZoneOffset {
        self.fixed(self.offset_at(utc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    fn utc(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    fn hours(zone: &str, at: &str) -> f64 {
        let z: Zone = zone.parse().unwrap();
        let off = utc(at).with_timezone(&z).offset().fix().local_minus_utc();
        f64::from(off) / 3600.0
    }

    /// One second before and at the instant of each 2026 change.
    fn around(zone: &str, change: &str, before: f64, after: f64) {
        let t = utc(change);
        let z: Zone = zone.parse().unwrap();
        let off = |d: DateTime<Utc>| {
            f64::from(d.with_timezone(&z).offset().fix().local_minus_utc()) / 3600.0
        };
        assert_eq!(
            off(t - Duration::seconds(1)),
            before,
            "{zone} before {change}"
        );
        assert_eq!(off(t), after, "{zone} at {change}");
    }

    #[test]
    fn transitions_of_2026() {
        around("America/New_York", "2026-03-08T07:00:00Z", -5.0, -4.0);
        around("America/New_York", "2026-11-01T06:00:00Z", -4.0, -5.0);
        around("America/St_Johns", "2026-03-08T05:30:00Z", -3.5, -2.5);
        around("Europe/London", "2026-03-29T01:00:00Z", 0.0, 1.0);
        around("Europe/London", "2026-10-25T01:00:00Z", 1.0, 0.0);
        around("Europe/Helsinki", "2026-03-29T01:00:00Z", 2.0, 3.0);
        around("Atlantic/Azores", "2026-10-25T01:00:00Z", 0.0, -1.0);
        around("America/Santiago", "2026-04-05T03:00:00Z", -3.0, -4.0);
        around("America/Santiago", "2026-09-06T04:00:00Z", -4.0, -3.0);
        around("Australia/Sydney", "2026-04-04T16:00:00Z", 11.0, 10.0);
        around("Australia/Sydney", "2026-10-03T16:00:00Z", 10.0, 11.0);
        around("Australia/Adelaide", "2026-10-03T16:30:00Z", 9.5, 10.5);
        around("Pacific/Auckland", "2026-04-04T14:00:00Z", 13.0, 12.0);
        around("Pacific/Auckland", "2026-09-26T14:00:00Z", 12.0, 13.0);
        around("Asia/Jerusalem", "2026-03-27T00:00:00Z", 2.0, 3.0);
        around("Asia/Jerusalem", "2026-10-24T23:00:00Z", 3.0, 2.0);
        around("Africa/Cairo", "2026-04-23T22:00:00Z", 2.0, 3.0);
        around("Africa/Cairo", "2026-10-29T21:00:00Z", 3.0, 2.0);
        around("Asia/Beirut", "2026-03-28T22:00:00Z", 2.0, 3.0);
        around("Asia/Beirut", "2026-10-24T21:00:00Z", 3.0, 2.0);
    }

    #[test]
    fn fixed_zones() {
        assert_eq!(hours("America/Sao_Paulo", "2026-01-15T12:00:00Z"), -3.0);
        assert_eq!(hours("America/Sao_Paulo", "2026-07-15T12:00:00Z"), -3.0);
        assert_eq!(hours("Asia/Kolkata", "2026-07-15T12:00:00Z"), 5.5);
        assert_eq!(hours("Asia/Kathmandu", "2026-07-15T12:00:00Z"), 5.75);
        assert_eq!(hours("UTC", "2026-07-15T12:00:00Z"), 0.0);
        assert!("Mars/Olympus".parse::<Zone>().is_err());
        assert!("america/sao_paulo".parse::<Zone>().is_err());
    }

    #[test]
    fn local_times_that_skip_or_repeat() {
        let ny: Zone = "America/New_York".parse().unwrap();
        let at = |s: &str| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").unwrap();
        // 02:30 does not exist on the day the clock jumps.
        assert!(matches!(
            ny.from_local_datetime(&at("2026-03-08 02:30")),
            MappedLocalTime::None
        ));
        // 01:30 happens twice on the day the clock goes back; the first one is the summer one.
        let MappedLocalTime::Ambiguous(a, b) = ny.from_local_datetime(&at("2026-11-01 01:30"))
        else {
            panic!("repeated hour");
        };
        assert_eq!(a.with_timezone(&Utc), utc("2026-11-01T05:30:00Z"));
        assert_eq!(b.with_timezone(&Utc), utc("2026-11-01T06:30:00Z"));
        let open = ny
            .from_local_datetime(&at("2026-07-01 09:30"))
            .single()
            .unwrap();
        assert_eq!(open.with_timezone(&Utc), utc("2026-07-01T13:30:00Z"));
    }

    #[test]
    fn names_are_unique() {
        let mut names: Vec<&str> = ZONES.iter().map(|z| z.name).collect();
        names.sort();
        let n = names.len();
        names.dedup();
        assert_eq!(n, names.len());
    }
}
