mod parser;
use parser::PowerShellParser;

type DynError = Box<dyn std::error::Error>;
type DynResult<T> = core::result::Result<T, DynError>;

fn main() -> DynResult<()> {
    let script = std::env::args().collect::<Vec<String>>()[0].clone();
    let deobfuscated = PowerShellParser::new().deobfuscate(script.as_str())?;
    println!("------------Deobfuscated------------\n\n {}", deobfuscated);
    Ok(())
}

pub fn safe_eval(script: &str) -> DynResult<String> {
    Ok(PowerShellParser::new().safe_eval(script)?)
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use super::*;

    #[test]
    fn obfuscation_1() {
        let input = r#"
$ilryNQSTt="System.$([cHAR]([ByTE]0x4d)+[ChAR]([byte]0x61)+[chAr](110)+[cHar]([byTE]0x61)+[cHaR](103)+[cHar](101*64/64)+[chaR]([byTE]0x6d)+[cHAr](101)+[CHAr]([byTE]0x6e)+[Char](116*103/103)).$([Char]([ByTe]0x41)+[Char](117+70-70)+[CHAr]([ByTE]0x74)+[CHar]([bYte]0x6f)+[CHar]([bytE]0x6d)+[ChaR]([ByTe]0x61)+[CHar]([bYte]0x74)+[CHAR]([byte]0x69)+[Char](111*26/26)+[chAr]([BYTe]0x6e)).$(('Âmsí'+'Ùtìl'+'s').NORmalizE([ChAR](44+26)+[chAR](111*9/9)+[cHar](82+32)+[ChaR](109*34/34)+[cHaR](68+24-24)) -replace [ChAr](92)+[CHaR]([BYTe]0x70)+[Char]([BytE]0x7b)+[CHaR]([BYTe]0x4d)+[chAR](110)+[ChAr](15+110))";$ilryNQSTt
"#;

        let mut p = PowerShellParser::new();
        assert_eq!(
            p.safe_eval(input).unwrap().as_str(),
            "System.Management.Automation.AmsiUtils"
        );
    }

    #[test]
    fn obfuscation_2() {
        let input = r#"
$(('W'+'r'+'î'+'t'+'é'+'Í'+'n'+'t'+'3'+'2').NormAlIzE([chaR]([bYTE]0x46)+[CHAR](111)+[ChAR]([Byte]0x72)+[CHAR]([BytE]0x6d)+[CHAr](64+4)) -replace [cHAr]([BytE]0x5c)+[char]([bYtE]0x70)+[ChAR]([byTe]0x7b)+[cHar]([bYtE]0x4d)+[Char]([bYte]0x6e)+[CHAR](125))
"#;

        let mut p = PowerShellParser::new();
        assert_eq!(p.safe_eval(input).unwrap().as_str(), "WriteInt32");
    }

    #[test]
    fn obfuscation_3() {
        let input = r#"
$([cHar]([BYte]0x61)+[chAr]([bYTE]0x6d)+[CHaR]([ByTe]0x73)+[char](105)+[CHAR]([bYTE]0x43)+[cHaR](111)+[chaR]([bYTE]0x6e)+[cHAr]([bYTe]0x74)+[cHAr](32+69)+[cHaR](120+30-30)+[cHAR]([bYte]0x74))
"#;

        let mut p = PowerShellParser::new();
        assert_eq!(p.safe_eval(input).unwrap().as_str(), "amsiContext");
    }

    #[test]
    fn obfuscation_4() {
        let input = r#"
"System.$(('Má'+'nã'+'gê'+'mê'+'nt').noRMaliZe([cHaR](42+28)+[cHar]([BYTe]0x6f)+[ChAr](114)+[CHar]([BYtE]0x6d)+[CHaR](42+26)) -replace [cHAR]([byTe]0x5c)+[chAr](109+3)+[chAr]([BytE]0x7b)+[chAR](77*19/19)+[CHar]([bYte]0x6e)+[cHAr]([bYte]0x7d)).$([chAR]([BYte]0x41)+[CHar](117+70-70)+[CHAr](116*55/55)+[cHAR](111)+[cHAR](109)+[chAr]([ByTe]0x61)+[chAR]([bYte]0x74)+[cHAR](105*26/26)+[CHAr]([BYtE]0x6f)+[CHar]([BYte]0x6e)).$([ChAr](65+1-1)+[CHAR](109*74/74)+[chaR]([ByTE]0x73)+[chAR](105+27-27)+[chAr]([byTe]0x55)+[CHar](116*94/94)+[cHar](25+80)+[cHaR](108+105-105)+[cHar]([BytE]0x73))"
"#;

        let mut p = PowerShellParser::new();
        assert_eq!(
            p.safe_eval(input).unwrap().as_str(),
            "System.Management.Automation.AmsiUtils"
        );
    }

    #[test]
    fn obfuscation_5() {
        let _ = env_logger::builder().filter_level(log::LevelFilter::max()).is_test(true).try_init();
        let input = r#"
$NS1R4CJUj8=[System.Runtime.InteropServices.Marshal]::AllocHGlobal((4316+4760));
"#;

        let mut p = PowerShellParser::new();
        println!("{}",PowerShellParser::new().deobfuscate(input).unwrap().as_str());
        assert_eq!(
            "",
            "System.Management.Automation.AmsiUtils"
        );
    }

    #[test]
    fn obfuscation_6() {
        let _ = env_logger::builder().filter_level(log::LevelFilter::max()).is_test(true).try_init();
        let input = r#"
#Unknown - Force error
$xqyne=$null;
$NS1R4CJUj8=[System.Runtime.InteropServices.Marshal]::AllocHGlobal((4316+4760));
$jklfxhgcmsztpjdnnevqyzeavjmq="+('r'+'í'+'g').NoRmaLize([ChAr](51+19)+[chAR](111)+[cHaR](35+79)+[chAr](109+26-26)+[chAR]([byte]0x44)) -replace [ChaR]([Byte]0x5c)+[chAr](57+55)+[char](8+115)+[chAR]([BytE]0x4d)+[cHar]([BYTE]0x6e)+[CHAr](125+72-72)";
[Threading.Thread]::Sleep(1425);
[Ref].Assembly.GetType("System.$([ChaR]([byTE]0x4d)+[CHAr](97*68/68)+[CHAr](110)+[Char](97+88-88)+[chAr]([byte]0x67)+[cHar]([Byte]0x65)+[CHar](109)+[cHAr](101*39/39)+[cHaR](110)+[chAr](116*13/13)).$([ChaR](58+7)+[CHar]([BYTE]0x75)+[CHaR](116+113-113)+[ChAR](111*47/47)+[char](109+9-9)+[cHar](75+22)+[Char]([bYtE]0x74)+[cHaR]([BYtE]0x69)+[CHaR]([BYtE]0x6f)+[cHar]([byTe]0x6e)).$([CHAr](65*49/49)+[CHAR]([BYTe]0x6d)+[CHar]([byte]0x73)+[CHAR](105)+[CHAR]([BYTE]0x55)+[chAR](116+14-14)+[cHAR]([ByTE]0x69)+[ChAR](108)+[cHaR](115))").GetField("$([CHAr]([BYtE]0x61)+[Char](109*3/3)+[CHAr](77+38)+[Char](72+33)+[cHAr](83+41-41)+[chAr](101)+[chaR]([BytE]0x73)+[cHAR](115*2/2)+[chAR]([BYte]0x69)+[char](39+72)+[chAr]([bYTE]0x6e))", "NonPublic,Static").SetValue($xqyne, $xqyne);
[Ref].Assembly.GetType("System.$([ChaR]([byTE]0x4d)+[CHAr](97*68/68)+[CHAr](110)+[Char](97+88-88)+[chAr]([byte]0x67)+[cHar]([Byte]0x65)+[CHar](109)+[cHAr](101*39/39)+[cHaR](110)+[chAr](116*13/13)).$([ChaR](58+7)+[CHar]([BYTE]0x75)+[CHaR](116+113-113)+[ChAR](111*47/47)+[char](109+9-9)+[cHar](75+22)+[Char]([bYtE]0x74)+[cHaR]([BYtE]0x69)+[CHaR]([BYtE]0x6f)+[cHar]([byTe]0x6e)).$([CHAr](65*49/49)+[CHAR]([BYTe]0x6d)+[CHar]([byte]0x73)+[CHAR](105)+[CHAR]([BYTE]0x55)+[chAR](116+14-14)+[cHAR]([ByTE]0x69)+[ChAR](108)+[cHaR](115))").GetField("$([chAr](97)+[cHar]([bytE]0x6d)+[CHaR]([BYTe]0x73)+[chaR](105*89/89)+[ChaR]([bYtE]0x43)+[chAR]([byTe]0x6f)+[cHAR](110)+[char]([byTE]0x74)+[char]([byTE]0x65)+[ChaR](34+86)+[Char](116*13/13))", "NonPublic,Static").SetValue($xqyne, [IntPtr]$NS1R4CJUj8);
$weszwpifqmkophvigibm="+('èmõxcjróbgvrkvhéró'+'khfnjpjp').noRmALIzE([CHaR]([ByTE]0x46)+[cHar](111*6/6)+[chAr]([ByTE]0x72)+[chAr](109)+[cHAR]([bYte]0x44)) -replace [cHAR]([Byte]0x5c)+[ChAR](100+12)+[chaR]([Byte]0x7b)+[ChAR](77+50-50)+[char](110+15-15)+[ChAr](125)"
"#;

        let mut p = PowerShellParser::new();
        assert_eq!(
            p.safe_eval(input).unwrap().as_str(),
            "System.Management.Automation.AmsiUtils"
        );
    }
}
