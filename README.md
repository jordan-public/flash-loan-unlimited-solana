# FLUF Protocol - Flash Loan Unlimited Facility on Solana

## Demo

See video [here](./demo/README.md) and on [YouTube here]().

## Abstract

A novel universal Flash Loan Facility is described and prototyped on Solana. With an unusual calling pattern, it allows
for Flash Loans in multiple protocols that are agnostic to and unaware of Flash Loans. In addition,
this facility allows for a novel mechanism in which unlimited non-existent funds can be borrowed for the duration
of a single transaction, to achieve even more powerful results than the typical flash loans.

## Introduction

Flash loans are a new lending mechanism typical to Decentralized Finance (DeFi). Protocols are able to
borrow funds for the duration of a single atomic transaction. The transaction can succeed only if the flash loan is
repaid within the same transaction along with the associated fee. 

Flash loans have allowed arbitrage, timely liquidations and other very useful operations to be performed by parties
with no sizable funds making DeFi a plain field available to anyone and more fair. This availability is making
the associated DeFi protocols more nimble and thus more stable, more liquid and price-adjusted against other markets.

Flash loans have been a tool for abuse as well, but that has generally occurred with protocols that are poorly written
and as such, subject to manipulation of price oracles. In addition, flash loans have been used to amplify exploits in buggy
protocols and maximize the attack effects.

One of the first protocols allowing this mechanism was Uniswap, which called it Flash Swaps:

![Uniswap Flash Swap](./Uniswap%20Flash%20Swap.png)

The purpose of of the
Flash Swaps was to perform price arbitrage and equalize the Uniswap price against other on-chain exchanges. To achieve
this, the sequence of swapping asset A for asset B was:
1. Optimistically receive the asset B (in counter-value of the asset A).
2. Having received asset B, while still in possession of asset A, call out another protocol (intentional callback). 
This would typically be a swap of asset B to A at another decentralized exchange (DEX) at better price.
3. Having received more A because of the other DEX's better price, pay the required counter-value of asset A, and
have some left-over of A to pay the fee and receive profit.

Note that the above Flash Swap can be executed by caller that has no funds (other than for gas), and make profit,
while importantly performing price arbitrage to equalize the appropriate Uniswap market (pool) with the other DEX.
Note that the borrowed funds have to previously exist in the appropriate Uniswap pool before calling the Flash Swap.

There is also an additional issue with Flash Loans: they are not available when they are needed the most. Namely,
in times of low liquidity, the sources of flash lending are insufficient due to market conditions.

Our calling pattern is different, as it originates at our protocol and ends with our protocol, thus allowing for more
universality.

In addition, it allows for **borrowing funds that do not exist**. This technique
has been proven in a typical calling pattern with single protocol and demonstrated in liquidation of a delinquent leveraged position,
otherwise not possible with usual flash loan, at [CFD DEX and Flash Trillions](https://github.com/jordan-public/cfd-dex)
presented at 
[ETHGlobal Scaling Ethereum 2023](https://ethglobal.com/showcase/cfd-dex-and-flash-trillions-ad666). 

However, while such flash loan of non-existent funds was demonstrated to work in a single protocol for liquidations,
**we go one step further**, allowing such Unlimited Flash Loans that borrow non-existent funds, to be used **across multiple protocols**.

Read on...

## Flash Loan Facility

In order to allow for universal Flash Loan Facility usable by sequence of protocols all unaware of the flash loans, we use a different calling pattern / sequence:

![Flash Loan Facility](./Flash%20Loan%20Facility.png)

1. The Flash Loan Facility borrows funds from its pool (PDA - Solana Program Derived Account) and transfers the borrowed funds to the user's program PDA.
2. The Flash Loan Facility calls (via CPI - Solana Cross Porgram Invocation) the user-written program intended for execution of the Flash Loan transaction.
3. Having funds available, the user-written program permissionlessly calls a sequence of protocols it wants to utilize.
4. Once the call to the user-written program returns, the Flash Loan Facility
checks whether the loan is repaid along with the fee and otherwise it reverts
the entire transaction.

But what does the "U" stand for in the FLUF Protocol? Read on...

## Unlimited Flash Loans - Borrowing Non-existent Funds

**FLUF** stands for **Flash Loan Unlimited Facility**. The FLUF Protocol can offer Flash Loans of **any** requested amount, regardless whether the funds exist or not. How does this work? Here is the explanation:

Protocols are usually agnostic to the assets used and anyone can create new pools. For example, in an AMM (Automated Market Maker) anyone can create a Liquidity Pool for trading any desired pair of tokens A and B. As appropriate
amounts of A and B are deposited in the Liquidity Pool, then anyone can trade
against the Liquidity Pool.

Let's say we wrap both tokens A and B with another pair of tokens wA and wB, in one-to-one proportion (each token wA contains/wraps one token A and each token wB contains/wraps one token B).

We can then create a Liquidity Pool for trading wA and wB and deposit wA and wB in it. This is equivalent in value to the original Liquidity Pool of A and B
as long as the amounts of wA and wB in the new pool are the same as the amounts of A and B in the original pool. To trade A for B in the new pool,
we can wrap A into wA, trade wA for wB and then unwrap B from the returned wB,
thus achieving the same as trading A for B in the original pool.

To recap:
- To **wrap** amount x of A into wA means to lock amount x of A, then to mint
amount x of wA and to return it to the caller.
-  To **unwrap** amount x of wA means to burn amount x of wA, then to unlock
amount x of A and return it to the caller.

The FLUF Protocol (Flash Loan **Unlimited** Facility Protocol) operates Pools of capital (tokens) and it is in charge of minting and burning but it enforces the following rules:

1. Any **investor** depositing a token **T** into the FLUF Pool, receives an equivalent value of amount of token **fT**. Note that the amount of fT received is not necessary the same amount of T deposited, as fT is an appreciating asset, as we will see that in the "Economics" section below. In addition, as the fT tokens contain actual collateralized value, they can be
used in other DeFi protocols, for additional yield, but that's irrelevant
to our explanation.
2. The FLUF Protocol operates the pools in two **Modes**: **Direct** and **Wrapped**, and this is **determined at the time of lending** by the parameter named "wrapped". In Direct Mode, it lends tokens **T**, but in the Wrapped Mode it lends tokens **wT**. Outside of Flash Loans each wT is equivalent to one T (it wraps 1 T). Note that for Unlimited Flash Loans the FLUF Protocol lends in Wrapped Mode.
3. Outside of Flash Loans, only **T** can be exchanged for **fT** (Pool Deposit) and vice versa (Pool Withdrawal).
4. Inside Flash Loan (entrypoint ```lendAndCall```), 
    - in Direct Mode, the amount of loan is limited to the amount of tokens T deposited in the pool. 
    - in Wrapped Mode, the FLUF Protocol **mints** **any** requested amount of wT without collateral (regardless of the amount of T in the pool). However, this amount has to be repaid by the end of the transaction along with the fee, but then the FLUF Protocol **burns** only the amount that was lent (and not the fee). The reason the FLUF Protocol can mint any amount without limit
    in Wrapped Mode is because it does so without collateral, yet with appropriate repayment accounting. 

![Uncollateralized Minting](./Umcollateralized%20Minting.png)

## Economics

The depositors (investors) in FLUF Pools make profit from the Pool's operation as follows:
- in Direct Mode, each Flash Loan has to be repaid (back to the Pool) in token T along with
the **fee** of **0.3%** of the original amount borrowed.
- in Wrapped Mode, each Flash Loan has to be repaid (back to the Pool) in token wT along
with the **fee** of **0.1%** of the original amount borrowed.

With the above we can see that the pool grows in value, which is the reason why 1 fT is not equal to 1 T. Each pool has a factor $f$. Then the following rules are enforced:
- When an investor deposits into the pool, the factor $f_d = f$ is recorded with the deposit. If there was a previous deposit of $d$ tokens T and depositing new $n$ tokens T, instead of rebasing the total deposit to: $d * f / f_d + n$, the factor $f_d$ is adjusted to achieve equivalence at the same
amount of deposit $d + n$: $f_d = f_d * (d+n) / (d * f / f_d + n)$.
- When a Flash Loan is repaid, it's factor **f** is adjusted as follows: $f = f * T_a / T_b$, where $T_b$ is the amount of T in the pool before the flash loan and $T_a$ is the amount of T in the pool after the flash loan.
- When the investor withdraws from the Pool by returning $x$ fT tokens, he
receives $x * f_w / f_d$ where $f_d$ is the Pool's factor $f$ recorded at the time with the investor's deposit, and $f_w$ is the curren Pool's factor $f$ (at the time of withdrawal).

The above calculations stimulate initial investors, but not unfairly. As there
is more need for wT tokens in circulation, there is more need for minting wT,
but also more usage of wT for all investors to enjoy.

So far only the yield obtained from the FLUF Protocol is explained. But there is more: As investors deposit tokens T into the FLUF Protocol
they receive tokens fT, which have
actual collateralized value. As such they can be used in other DeFi protocols
for additional yield, for example to borrow other assets against fT, or
even to borrow more T against fT and re-engage the borrowed T in another round
of investment into the FLUF Protocol. Moreover, borrowing T against fT should
not require much overcollateralization, as fT is an appreciating asset relative
to T and there is no Delta Risk.

Here are some important questions and answers, which clarify the economics
of the Pool's operation:
- Q: Why do we need tokens wT in Wrapped Mode outside of Flash Loans?
- A: Because they have actual collateralized value, for other protocols to
trade and the FLUF Protocol to receive fees.

--
- Q: Why can then wT be minted in any amount in Wrapped Mode inside Flash Loans? Isn't that frivolous?
- A: It is not frivolous because whatever is minted has to be burned at the
end in the same amount. No one profits and no one loses. This can be viewed
as stimulus in order to perform arbitrage, liquidate delinquent derivative
positions, liquidate delinquent loans etc.

## Multi-protocol Use

## Implementation and Integration Instructions

## Conclusion

